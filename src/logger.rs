use hyper::{Request, body::Incoming, service::Service};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Logger<S> {
    inner: S,
}
impl<S> Logger<S> {
    #[allow(unused)]
    pub fn new(inner: S) -> Self {
        Logger { inner }
    }
}

type Req = Request<Incoming>;

impl<S> Service<Req> for Logger<S>
where
    S: Service<Req>,
{
    type Response = S::Response;
    type Future = S::Future;
    type Error = S::Error;
    fn call(&self, req: Req) -> Self::Future {
        println!("processing request: {} {}", req.method(), req.uri().path());
        self.inner.call(req)
    }
}
