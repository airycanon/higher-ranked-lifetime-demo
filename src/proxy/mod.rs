use futures::future::BoxFuture;
use http::{Request, Response};
use std::fmt::{Debug, Error};

pub mod chain;
pub mod handlers;

#[derive(Debug)]
pub enum HttpResult<B> {
    /// HTTP Request
    Request(Request<B>),
    /// HTTP Response
    Response(Response<B>),
}

type Result<T> = std::result::Result<T, Error>;

pub trait HttpHandler<B>: Send + Sync + 'static
where
    B: Send + Debug + 'static,
{
    fn handle_request(&self, request: Request<B>) -> BoxFuture<'static, Result<HttpResult<B>>>;

    fn handle_response(&self, request: Response<B>) -> BoxFuture<'static, Result<Response<B>>>;
}
