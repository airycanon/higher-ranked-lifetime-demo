use crate::proxy::HttpHandler;
use crate::proxy::{HttpResult, Result};
use axum::body::Body as ReverseBody;
use axum::extract::State;
use axum::response::IntoResponse;
use futures::future::BoxFuture;
use http::{Request, Response};
use hudsucker::{Body as ForwardBody, HttpContext, RequestOrResponse};
use hyper_util::client::legacy::connect::HttpConnector;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

type Client = hyper_util::client::legacy::Client<HttpConnector, ReverseBody>;
pub struct Chain<B, H>
where
    B: Send + Debug + 'static,
    H: HttpHandler<B> + ?Sized + Send + Sync + 'static,
{
    handlers: Vec<Arc<H>>,
    phantom: PhantomData<B>,
}

impl<B, H> Chain<B, H>
where
    B: Send + Debug + 'static,
    H: HttpHandler<B> + ?Sized + Send + Sync + 'static,
{
    pub fn new(handlers: Vec<Arc<H>>) -> Self {
        Self {
            handlers,
            phantom: PhantomData,
        }
    }

    async fn process_request(&self, request: Request<B>) -> Result<HttpResult<B>> {
        let handlers = self.handlers.clone();
        let mut current = request;
        for handler in handlers {
            match handler.handle_request(current).await? {
                HttpResult::Request(req) => current = req,
                response => return Ok(response),
            }
        }
        Ok(HttpResult::Request(current))
    }

    async fn process_response(&self, response: Response<B>) -> Result<Response<B>> {
        let handlers = self.handlers.clone();
        let mut current = response;
        for handler in handlers {
            current = handler.handle_response(current).await?;
        }
        Ok(current)
    }
}

impl<B, H> Clone for Chain<B, H>
where
    B: Send + Debug + 'static,
    H: HttpHandler<B> + ?Sized + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handlers: self.handlers.clone(),
            phantom: PhantomData,
        }
    }
}

unsafe impl<B, H> Sync for Chain<B, H>
where
    B: Send + Debug + 'static,
    H: HttpHandler<B> + ?Sized + Send + Sync + 'static,
{
}

pub type ReverseChain = Chain<ReverseBody, dyn HttpHandler<ReverseBody>>;
pub type ForwardChain = Chain<ForwardBody, dyn HttpHandler<ForwardBody>>;

impl hudsucker::HttpHandler for ForwardChain {
    fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        request: Request<ForwardBody>,
    ) -> impl Future<Output = RequestOrResponse> + Send {
        let chain = self.clone();
        async move {
            match chain.process_request(request).await {
                Ok(HttpResult::Request(request)) => RequestOrResponse::Request(request),
                Ok(HttpResult::Response(response)) => RequestOrResponse::Response(response),
                Err(_) => RequestOrResponse::Response(Response::new(ForwardBody::empty())),
            }
        }
    }

    fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        response: Response<ForwardBody>,
    ) -> impl Future<Output = Response<ForwardBody>> + Send {
        let chain = self.clone();
        async move {
            chain.process_response(response).await.unwrap_or_else(|_| Response::new(ForwardBody::empty()))
        }
    }
}

impl<T> axum::handler::Handler<T, State<Client>> for ReverseChain {
    type Future = BoxFuture<'static, Response<ReverseBody>>;

    fn call(self, request: Request<ReverseBody>, State(client): State<Client>) -> Self::Future {
        let chain = self.clone();

        Box::pin(async move {
            let response = match chain.process_request(request).await {
                Ok(HttpResult::Request(request)) => match client.request(request).await {
                    Ok(response) => response.into_response(),
                    Err(_) => return Response::default(),
                },
                Ok(HttpResult::Response(response)) => response,
                Err(_) => Response::new(ReverseBody::empty()),
            };

            chain.process_response(response).await.unwrap_or_else(|_| Response::new(ReverseBody::empty()))
        })
    }
}
