use hyper::Response;
use tower_http::trace::{MakeSpan, OnResponse};
use tracing::Span;

#[derive(Clone)]
pub struct MyOnResponse {}

impl<B> OnResponse<B> for MyOnResponse {
    fn on_response(
        self,
        response: &Response<B>,
        latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        tracing::info!(
            latency = latency.as_millis(),
            status = response.status().as_u16(),
            "response"
        )
    }
}

#[derive(Clone)]
pub struct MyMakeSpan {}

impl<B> MakeSpan<B> for MyMakeSpan {
    fn make_span(&mut self, request: &hyper::Request<B>) -> Span {
        tracing::info_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
        )
    }
}
