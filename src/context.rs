use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};

/// Holds the incoming request and helpers for building responses.
pub struct Context {
    pub request: Request<Incoming>,
}

impl Context {
    pub(crate) fn new(request: Request<Incoming>) -> Self {
        Self { request }
    }

    /// Respond with a plain-text body.
    pub fn string(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from(body.into())))
            .expect("failed to build response")
    }

    /// Respond with a JSON body.
    pub fn json(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(Full::new(Bytes::from(body.into())))
            .expect("failed to build response")
    }

    /// Respond with an HTML body.
    pub fn html(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(body.into())))
            .expect("failed to build response")
    }

    /// Return the request path.
    pub fn path(&self) -> &str {
        self.request.uri().path()
    }

    /// Return the HTTP method.
    pub fn method(&self) -> &Method {
        self.request.method()
    }
}
