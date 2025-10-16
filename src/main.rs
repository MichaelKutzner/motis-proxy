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

mod config;
mod logger;

async fn proxy(req: Request<Incoming>) -> Result<Response<Incoming>, Infallible> {
    let config = config::Config::load();

    let stream = TcpStream::connect(config.backend_address.clone())
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
    let next = q.strip_prefix(config.subpath.as_str()).unwrap_or_else(|| {
        println!(
            "WARNING Path does not match subpath '{}': '{}'",
            config.subpath, q
        );
        q
    });
    let url = format!(
        "http://{}{}?{}",
        config.backend_address,
        next,
        req.uri().query().unwrap_or("")
    );
    let proxy_req = Request::builder()
        .uri(url)
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
    let config = config::Config::load();
    println!("Used config: {:?}", config);
    let addr = SocketAddr::from(config.bind_addr);
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
