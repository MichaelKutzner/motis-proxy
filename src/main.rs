use http_body_util::Empty;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    server::conn::http1,
};
use hyper_util::rt::TokioIo;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceBuilder;

mod logger;

async fn proxy(req: Request<Incoming>) -> Result<Response<Incoming>, Infallible> {
    const BACKEND: &str = "http://127.0.0.1:8080";
    let backend_url = BACKEND.parse::<hyper::Uri>().expect("URI parsing failed");
    let host = backend_url.host().expect("BACKEND has no host");
    let port = backend_url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    let authority = backend_url.authority().unwrap().clone();

    let stream = TcpStream::connect(address)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .expect("Handshake failed");
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let q = req.uri().path();
    const PREFIX: &str = "/motis";
    let next = q.strip_prefix(PREFIX).unwrap_or_else(|| {
        println!("WARNING Incorrect path: '{}'", q);
        q
    });
    let url = format!(
        "http://{}:{}{}?{}",
        host,
        port,
        next,
        req.uri().query().unwrap_or("")
    );
    let proxy_req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())
        .expect("Failed to build proxy request");

    let res = sender
        .send_request(proxy_req)
        .await
        .expect("Request failed");
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 5173));
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            let svc = hyper::service::service_fn(proxy);
            let svc = ServiceBuilder::new()
                // .layer_fn(logger::Logger::new)
                .service(svc);
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("server error: {}", err);
            }
        });
    }
}
