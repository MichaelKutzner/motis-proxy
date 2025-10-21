use hyper::{
    Request, Response, body::Incoming, client::conn::http1::SendRequest, server::conn::http1,
};
use hyper_util::rt::TokioIo;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceBuilder;

mod config;
mod path_rewriter;
mod time;

async fn forward_request(
    req: Request<Incoming>,
    backend_server: &String,
) -> Result<Response<Incoming>, Infallible> {
    Ok(connect_to_backend(backend_server)
        .await
        .expect("Connection to backend server failed")
        .send_request(req)
        .await
        .expect("Request failed"))
}

async fn connect_to_backend(backend_server: &String) -> Result<SendRequest<Incoming>, Infallible> {
    let stream = TcpStream::connect(backend_server)
        .await
        .expect("Failed to connect to the backend server");
    let io = TokioIo::new(stream);
    let (sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .expect("Handshake failed");
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    Ok(sender)
}

async fn proxy(
    req: Request<Incoming>,
    config: Arc<config::Config>,
) -> Result<Response<Incoming>, Infallible> {
    let current_day_offset = time::get_current_day_offset(&req);
    // println!("Request starts in {:?} days", current_day_offset);
    forward_request(req, &config.default_backend_address).await
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
        let service_config = config.clone();
        tokio::spawn(async move {
            let svc = hyper::service::service_fn(|req| proxy(req, service_config.clone()));
            let svc = ServiceBuilder::new()
                .layer_fn(|inner| path_rewriter::PathRewriter::new(inner, service_config.clone()))
                .service(svc);
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("server error: {}", err);
            }
        });
    }
}
