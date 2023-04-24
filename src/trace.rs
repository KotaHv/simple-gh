use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::http::{header, Request, Response};
use futures_util::ready;
use pin_project::pin_project;
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing_subscriber::fmt::{self, time};
use yansi::Paint;

use crate::config::CONFIG;
use crate::util;

pub fn init() {
    fmt::fmt()
        .with_timer(time::LocalTime::rfc_3339())
        .with_max_level(CONFIG.log.level)
        .init();
}

#[derive(Clone)]
pub struct TraceLayer;

impl<S> Layer<S> for TraceLayer {
    type Service = TraceMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct TraceMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TraceMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TraceFuture<S::Future>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let ip = util::get_ip(&req);
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let headers = req.headers();
        let referer = util::get_header(&headers, header::REFERER).unwrap_or("-".to_string());
        let ua = util::get_ua(&headers);
        let response_future = self.inner.call(req);
        TraceFuture {
            response_future,
            ip,
            method,
            path,
            referer,
            start,
            ua,
        }
    }
}

#[pin_project]
pub struct TraceFuture<F> {
    #[pin]
    response_future: F,
    start: Instant,
    ip: String,
    method: String,
    path: String,
    referer: String,
    ua: String,
}

impl<F, ResBody, E> Future for TraceFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = ready!(this.response_future.poll(cx)?);
        if this.path.as_str() == "/alive" {
            return Poll::Ready(Ok(res));
        }
        if res.status().is_success() {
            info!(
                "{ip} {method} {path} => {status} \"{referer}\" {elapsed:?}",
                ip = Paint::cyan(this.ip),
                method = Paint::green(this.method),
                path = Paint::blue(this.path),
                status = Paint::yellow(res.status()),
                referer = this.referer,
                elapsed = this.start.elapsed()
            );
        } else {
            warn!(
                "{ip} {method} {path} => {status} \"{referer}\" [{ua}] {elapsed:?}",
                ip = Paint::red(this.ip).bold(),
                method = Paint::green(this.method),
                path = Paint::red(this.path).underline(),
                status = Paint::red(res.status()),
                referer = this.referer,
                elapsed = this.start.elapsed(),
                ua = Paint::magenta(this.ua),
            )
        }
        Poll::Ready(Ok(res))
    }
}
