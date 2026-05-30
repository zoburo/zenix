//! Request extraction framework — the trait and helpers behind
//! `ctx.bind::<T>()` and `#[zenix(from = "...")]`.
//!
//! # Trait
//!
//! - [`FromRequest`] — implemented by the `#[derive(FromRequest)]` macro.

use std::collections::HashMap;

use crate::context::Context;
use bytes::Bytes;
use http_body_util::Full;
use hyper::StatusCode;

/// Trait for types that can be extracted from the incoming request via
/// [`Context::bind`].
///
/// Normally you **derive** this rather than implementing it manually:
///
/// ```ignore
/// use zenix::FromRequest;
///
/// #[derive(FromRequest)]
/// struct CreateUser {
///     #[zenix(from = "body")]
///     name: String,
/// }
/// ```
pub trait FromRequest: Sized {
    /// Attempt to extract `Self` from the request context.
    fn from_request(ctx: &Context) -> Result<Self, crate::handler::Response>;
}

/// Build a `400 Bad Request` plain-text response.
pub fn bad_request(msg: impl Into<String>) -> crate::handler::Response {
    hyper::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Full::new(Bytes::from(msg.into())))
        .expect("static response")
}

/// Build a `422 Unprocessable Entity` JSON response from a validation report.
pub fn validation_error(report: &crate::validate::Report) -> crate::handler::Response {
    let body = crate::validate::ReportExt::to_json(report);
    hyper::Response::builder()
        .status(StatusCode::UNPROCESSABLE_ENTITY)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(Full::new(Bytes::from(body)))
        .expect("static response")
}

/// Parse the request body as a JSON [`serde_json::Value`].
pub fn parse_body_json(ctx: &Context) -> Result<serde_json::Value, crate::handler::Response> {
    let bytes = ctx.body();
    serde_json::from_slice(&bytes).map_err(|e| bad_request(format!("invalid JSON body: {e}")))
}

/// Parse the query string into a `HashMap<String, String>`.
pub fn parse_query(ctx: &Context) -> HashMap<String, String> {
    let query_str = ctx.query_string();
    // Simple URL-decoding parser for the query string
    let mut map = HashMap::new();
    for pair in query_str.split('&').filter(|s| !s.is_empty()) {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let key = url_decode(key);
            let val = parts.next().map(url_decode).unwrap_or_default();
            map.insert(key, val);
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    // Basic percent-decoding (+ → space, %XX → char)
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                let hi = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
                let lo = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
                result.push(char::from((hi * 16 + lo) as u8));
            }
            _ => result.push(c),
        }
    }
    result
}
