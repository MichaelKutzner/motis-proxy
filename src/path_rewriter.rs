use std::str::FromStr as _;

use hyper::{Request, body::Incoming, service::Service};
use std::sync::Arc;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct PathRewriter<S> {
    inner: S,
    config: Arc<Config>,
}
impl<S> PathRewriter<S> {
    pub fn new(inner: S, config: Arc<Config>) -> Self {
        PathRewriter { inner, config }
    }
}

type Req = Request<Incoming>;

impl<S> Service<Req> for PathRewriter<S>
where
    S: Service<Req>,
{
    type Response = S::Response;
    type Future = S::Future;
    type Error = S::Error;
    fn call(&self, mut req: Req) -> Self::Future {
        let request_path = req.uri().path_and_query().expect("Missing path").as_str();
        let backend_path = request_path
            .strip_prefix(self.config.subpath.as_str())
            .unwrap_or_else(|| {
                println!(
                    "WARNING Path does not match subpath '{}': '{}'",
                    self.config.subpath, request_path
                );
                request_path
            });
        *req.uri_mut() = hyper::Uri::from_str(backend_path).expect("Uri::from_str failed");
        self.inner.call(req)
    }
}
