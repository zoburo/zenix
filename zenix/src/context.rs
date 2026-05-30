use bytes::Bytes;
use http::{HeaderMap, Method, Uri};
use http_body_util::Full;
use hyper::{Response, StatusCode};

/// Holds the incoming request data (body pre-read) and helpers for building
/// responses.
pub struct Context {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    body: Bytes,
}

impl Context {
    pub(crate) fn new(
        method: Method,
        uri: Uri,
        headers: HeaderMap,
        body: Bytes,
    ) -> Self {
        Self {
            method,
            uri,
            headers,
            body,
        }
    }

    fn build(code: StatusCode, body: Full<Bytes>, content_type: &str) -> Response<Full<Bytes>> {
        Response::builder()
            .status(code)
            .header("Content-Type", content_type)
            .body(body)
            .expect("failed to build response")
    }

    /// Respond with a plain-text body (HTTP 200).
    pub fn string(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        Self::build(
            StatusCode::OK,
            Full::new(Bytes::from(body.into())),
            "text/plain; charset=utf-8",
        )
    }

    /// Respond with a JSON body (HTTP 200).
    pub fn json(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        Self::build(
            StatusCode::OK,
            Full::new(Bytes::from(body.into())),
            "application/json; charset=utf-8",
        )
    }

    /// Respond with an HTML body (HTTP 200).
    pub fn html(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        Self::build(
            StatusCode::OK,
            Full::new(Bytes::from(body.into())),
            "text/html; charset=utf-8",
        )
    }

    /// Return the request path.
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Return the HTTP method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Return a reference to the pre-read request body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Return the raw query string (the part after `?` in the URI).
    pub fn query_string(&self) -> &str {
        self.uri.query().unwrap_or("")
    }

    /// Deserialise the request into `T` using the [`FromRequest`] trait
    /// (powered by `#[derive(FromRequest)]`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(FromRequest)]
    /// struct Input {
    ///     #[zenix(from = "body")]
    ///     name: String,
    /// }
    ///
    /// fn handler(ctx: Context) -> Response {
    ///     let input: Input = match ctx.bind() {
    ///         Ok(v) => v,
    ///         Err(resp) => return resp,
    ///     };
    ///     ctx.json(format!("hello {}", input.name))
    /// }
    /// ```
    pub fn bind<T: crate::extract::FromRequest>(&self) -> Result<T, crate::handler::Response> {
        T::from_request(self)
    }

    /// Start building a response with a custom status code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ctx.status(StatusCode::UNPROCESSABLE_ENTITY)
    ///     .json(r#"{"error":"validation failed"}"#)
    /// ```
    pub fn status(self, code: StatusCode) -> StatusResponder {
        StatusResponder { code }
    }
}

// ── StatusResponder ──────────────────────────────────────────────────────────

/// Builder returned by [`Context::status`] that lets you set a custom HTTP
/// status code on the response.
pub struct StatusResponder {
    code: StatusCode,
}

impl StatusResponder {
    fn build(self, body: Full<Bytes>, content_type: &str) -> Response<Full<Bytes>> {
        Response::builder()
            .status(self.code)
            .header("Content-Type", content_type)
            .body(body)
            .expect("failed to build response")
    }

    /// Respond with a plain-text body using the configured status code.
    pub fn string(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        self.build(
            Full::new(Bytes::from(body.into())),
            "text/plain; charset=utf-8",
        )
    }

    /// Respond with a JSON body using the configured status code.
    pub fn json(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        self.build(
            Full::new(Bytes::from(body.into())),
            "application/json; charset=utf-8",
        )
    }

    /// Respond with an HTML body using the configured status code.
    pub fn html(self, body: impl Into<String>) -> Response<Full<Bytes>> {
        self.build(
            Full::new(Bytes::from(body.into())),
            "text/html; charset=utf-8",
        )
    }
}
