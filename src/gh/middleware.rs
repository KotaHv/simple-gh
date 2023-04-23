use std::task::{Context, Poll};

use super::CONFIG;
use axum::{
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use serde::Deserialize;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TokenLayer;

impl<S> Layer<S> for TokenLayer {
    type Service = TokenMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokenMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct TokenMiddleware<S> {
    inner: S,
}

#[derive(Debug, Deserialize)]
struct GHParams {
    token: String,
}

impl<S, T> Service<Request<T>> for TokenMiddleware<S>
where
    S: Service<Request<T>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<T>) -> Self::Future {
        let mut is_ok = false;
        if let Some(params) = request.uri().query() {
            if let Ok(params) = serde_urlencoded::from_str::<GHParams>(params) {
                if Some(params.token) == CONFIG.token {
                    is_ok = true;
                }
            }
        }
        let future = self.inner.call(request);
        Box::pin(async move {
            match is_ok {
                true => {
                    let response: Response = future.await?;
                    Ok(response)
                }
                false => Ok(StatusCode::NOT_FOUND.into_response()),
            }
        })
    }
}
