use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request, StatusCode, body::Incoming};
use hyper_util::rt::TokioIo;
use std::{
    collections::HashMap, convert::Infallible, future::Future, net::SocketAddr, pin::Pin, sync::Arc,
};
use tokio::net::TcpListener;

// ── Public re-exports ────────────────────────────────────────────────────────

pub use hyper::Response;

// ── Types ────────────────────────────────────────────────────────────────────

type BoxFuture = Pin<Box<dyn Future<Output = Response<Full<Bytes>>> + Send>>;
type Handler = Arc<dyn Fn(Context) -> BoxFuture + Send + Sync>;

// ── HandlerFn trait ───────────────────────────────────────────────────────────

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

// ── Context ──────────────────────────────────────────────────────────────────

/// Holds the incoming request and helpers for building responses.
pub struct Context {
    pub request: Request<Incoming>,
}

impl Context {
    fn new(request: Request<Incoming>) -> Self {
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

// ── Route key ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct RouteKey {
    method: Method,
    path: String,
}

// ── Server ───────────────────────────────────────────────────────────────────

/// A minimal async HTTP server.
pub struct Server {
    routes: HashMap<RouteKey, Handler>,
}

impl Server {
    /// Create a new, empty server.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn add_route<M, F>(&mut self, method: Method, path: &str, handler: F)
    where
        F: HandlerFn<M>,
    {
        let key = RouteKey {
            method,
            path: path.to_string(),
        };
        let handler = Arc::new(handler);
        self.routes
            .insert(key, Arc::new(move |ctx| Arc::clone(&handler).call(ctx)));
    }

    /// Register a GET handler.
    pub fn get<M, F>(&mut self, path: &str, handler: F)
    where
        F: HandlerFn<M>,
    {
        self.add_route(Method::GET, path, handler);
    }

    /// Register a POST handler.
    pub fn post<M, F>(&mut self, path: &str, handler: F)
    where
        F: HandlerFn<M>,
    {
        self.add_route(Method::POST, path, handler);
    }

    /// Register a PUT handler.
    pub fn put<M, F>(&mut self, path: &str, handler: F)
    where
        F: HandlerFn<M>,
    {
        self.add_route(Method::PUT, path, handler);
    }

    /// Register a DELETE handler.
    pub fn delete<M, F>(&mut self, path: &str, handler: F)
    where
        F: HandlerFn<M>,
    {
        self.add_route(Method::DELETE, path, handler);
    }

    /// Bind to `addr` (e.g. `":3000"`) and start serving.
    pub async fn listen(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = if addr.starts_with(':') {
            format!("0.0.0.0{}", addr)
        } else {
            addr.to_string()
        };

        let socket_addr: SocketAddr = addr.parse()?;
        let listener = TcpListener::bind(socket_addr).await?;

        println!("🚀 Listening on http://{}", socket_addr);

        let routes = Arc::new(self.routes);

        loop {
            let (stream, _peer) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let routes = Arc::clone(&routes);

            tokio::spawn(async move {
                let svc = hyper::service::service_fn(move |req: Request<Incoming>| {
                    let routes = Arc::clone(&routes);
                    async move {
                        let key = RouteKey {
                            method: req.method().clone(),
                            path: req.uri().path().to_string(),
                        };

                        let response = if let Some(handler) = routes.get(&key) {
                            let ctx = Context::new(req);
                            handler(ctx).await
                        } else {
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .header("Content-Type", "text/plain")
                                .body(Full::new(Bytes::from("404 Not Found")))
                                .unwrap()
                        };

                        Ok::<_, Infallible>(response)
                    }
                });

                if let Err(e) = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .serve_connection(io, svc)
                .await
                {
                    eprintln!("connection error: {}", e);
                }
            });
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
