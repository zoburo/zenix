use hyper::Method;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct RouteKey {
    pub method: Method,
    pub path: String,
}
