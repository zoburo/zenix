use crate::context::Context;
use bytes::Bytes;
use http_body_util::Full;
use hyper::Response;
use std::{future::Future, pin::Pin, sync::Arc};

pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = Response<Full<Bytes>>> + Send>>;
pub(crate) type Handler = Arc<dyn Fn(Context) -> BoxFuture + Send + Sync>;

/// Sealed trait implemented for both sync handlers (`FnOnce(Context) -> Response`)
/// and async handlers (`FnOnce(Context) -> impl Future<Output = Response>`).
///
/// We use a wrapper newtype approach to avoid orphan / blanket impl conflicts.
pub trait HandlerFn<Marker>: Send + Sync + 'static {
    fn call(self: Arc<Self>, ctx: Context) -> BoxFuture;
}

// Marker types to differentiate the two impls
pub struct SyncMarker;
pub struct AsyncMarker;

impl<F> HandlerFn<SyncMarker> for F
where
    F: Fn(Context) -> Response<Full<Bytes>> + Send + Sync + 'static,
{
    fn call(self: Arc<Self>, ctx: Context) -> BoxFuture {
        let resp = (self)(ctx);
        Box::pin(std::future::ready(resp))
    }
}

impl<F, Fut> HandlerFn<AsyncMarker> for F
where
    F: Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response<Full<Bytes>>> + Send + 'static,
{
    fn call(self: Arc<Self>, ctx: Context) -> BoxFuture {
        Box::pin((self)(ctx))
    }
}
