use crate::context::Context;
use bytes::Bytes;
use http_body_util::Full;
use std::sync::Arc;

/// The response type returned by every route handler.
pub type Response = hyper::Response<Full<Bytes>>;

/// Type-erased route handler.
pub(crate) type Handler = Arc<dyn Fn(Context) -> Response + Send + Sync>;
