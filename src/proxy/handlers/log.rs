use crate::proxy::{HttpHandler, HttpResult, Result};
use hudsucker::hyper::{Request, Response};
use std::fmt::Debug;
use std::marker::PhantomData;
use futures::future::BoxFuture;

#[derive(Debug)]
pub struct LogHandler<B>
where
    B: Send + Debug + 'static,
{
    phantom_data: PhantomData<B>,
}

impl<B> LogHandler<B>
where
    B: Send + Debug + 'static,
{
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

impl<B> HttpHandler<B> for LogHandler<B>
where
    B: Debug + Send + 'static,
{
    fn handle_request(
        &self,
        _request: Request<B>,
    ) -> BoxFuture<'static, Result<HttpResult<B>>> {
        todo!()
    }

    fn handle_response(
        &self,
        _request: Response<B>,
    ) -> BoxFuture<'static, Result<Response<B>>> {
        todo!()
    }
}

unsafe impl<B> Sync for LogHandler<B> where B: Send + Debug + 'static {}