use std::time::Duration;

use axum::http::{header, Request, Response};
use tower_http::{
    classify::{ClassifiedResponse, ClassifyResponse, NeverClassifyEos, SharedClassifier},
    trace::{MakeSpan, OnResponse, TraceLayer},
};
use tracing::Span;
use tracing_subscriber::fmt::{self, time};
use yansi::Paint;

use crate::config::CONFIG;
use crate::util;

pub fn init() {
    fmt::fmt()
        .with_timer(time::LocalTime::rfc_3339())
        .with_max_level(CONFIG.log.level)
        .pretty()
        .init();
}

pub fn layer(
) -> TraceLayer<SharedClassifier<MyClassifier>, MyMakeSpan, (), MyOnResponse, (), (), ()> {
    TraceLayer::new(SharedClassifier::new(MyClassifier))
        .make_span_with(MyMakeSpan)
        .on_response(MyOnResponse)
        .make_span_with(MyMakeSpan)
        .on_request(())
        .on_response(MyOnResponse)
        .on_body_chunk(())
        .on_eos(())
        .on_failure(())
}

#[derive(Copy, Clone)]
pub struct MyClassifier;

impl ClassifyResponse for MyClassifier {
    type FailureClass = String;
    type ClassifyEos = NeverClassifyEos<Self::FailureClass>;

    fn classify_response<B>(
        self,
        res: &Response<B>,
    ) -> ClassifiedResponse<Self::FailureClass, Self::ClassifyEos> {
        if res.status().is_success() {
            ClassifiedResponse::Ready(Ok(()))
        } else {
            ClassifiedResponse::Ready(Err(res.status().to_string()))
        }
    }

    fn classify_error<E>(self, error: &E) -> Self::FailureClass
    where
        E: std::fmt::Display + 'static,
    {
        error.to_string()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MyMakeSpan;

impl<B> MakeSpan<B> for MyMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let path = request.uri().path();
        if path == "/alive" {
            return trace_span!("request");
        }
        let path = match request.uri().query() {
            Some(query) => format!("{}?{}", path, query),
            None => path.to_string(),
        };
        let ip = Paint::cyan(util::get_ip(request));
        let method = Paint::green(request.method());
        let path = Paint::blue(path);
        let headers = request.headers();
        let referer = util::get_header(&headers, header::REFERER).unwrap_or("-".to_string());
        let ua = util::get_ua(&headers);
        info_span!("request", ip = ?ip, method = ?method, path=?path, referer = referer, user_agent=ua)
    }
}

#[derive(Clone, Debug)]
pub struct MyOnResponse;

impl<B> OnResponse<B> for MyOnResponse {
    fn on_response(self, res: &Response<B>, latency: Duration, span: &Span) {
        if !span.has_field("ip") {
            return;
        }

        let latency = Paint::white(latency);
        let status = format!("status={}", res.status());
        if res.status().is_success() {
            let status = Paint::yellow(status);
            info!("{status} latency={latency:?}");
        } else {
            let status = Paint::red(status);
            warn!("{status} latency={latency:?}");
        }
    }
}
