use hyper::{
    Request, Response, body::Incoming, client::conn::http1::SendRequest, server::conn::http1,
};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceBuilder;

mod config;
mod path_rewriter;
mod time;

async fn forward_request(
    req: Request<Incoming>,
    backend_server: &String,
) -> Result<Response<Incoming>, ErrorBox> {
    // println!("Sending request to '{}'", backend_server);
    Ok(connect_to_backend(backend_server)
        .await?
        .send_request(req)
        .await?)
}

async fn connect_to_backend(backend_server: &String) -> Result<SendRequest<Incoming>, ErrorBox> {
    let stream = TcpStream::connect(backend_server).await?;
    let io = TokioIo::new(stream);
    let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
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
) -> Result<Response<Incoming>, ErrorBox> {
    // println!("Request starts in {:?} days", current_day_offset);
    if let Some(current_day_offset) = time::get_current_day_offset(&req) {
        for backend in &config.backends {
            if backend.covers(current_day_offset) {
                return forward_request(req, &backend.backend_address).await;
            }
        }
    }
    forward_request(req, &config.default_backend_address).await
}

#[tokio::main]
async fn main() -> Result<(), ErrorBox> {
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

type ErrorBox = Box<dyn std::error::Error + Send + Sync>;
