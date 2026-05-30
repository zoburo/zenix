use crate::{
    context::Context,
    handler::{Handler, HandlerFn},
    route::RouteKey,
};
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper_util::rt::TokioIo;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

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
