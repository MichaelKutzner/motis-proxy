use hyper::{
    Request, Response, body::Incoming, client::conn::http1::SendRequest, server::conn::http1,
};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceBuilder;

use crate::current_offset::{get_offset_from_now, get_offset_from_timestamp};

mod config;
mod current_offset;
mod parameters;
mod path_rewriter;

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
    match parameters::parse_parameters(&req) {
        parameters::SearchParameters::Timestamp {
            timestamp,
            direction,
        } => send_to_first_backend_from_timestamp(req, config, timestamp, direction).await,
        parameters::SearchParameters::Now { direction } => {
            send_to_first_backend_from_now(req, config, direction).await
        }
        parameters::SearchParameters::Unrestricted | parameters::SearchParameters::None => {
            forward_request(req, &config.default_backend_address).await
        }
    }
}

async fn send_to_first_backend_from_timestamp(
    req: Request<Incoming>,
    config: Arc<config::Config>,
    timestamp: parameters::Timestamp,
    direction: parameters::SearchDirection,
) -> Result<Response<Incoming>, ErrorBox> {
    let max_duration_hours = config.max_duration_hours;
    send_to_first_backend(
        req,
        config,
        get_offset_from_timestamp(timestamp, direction, max_duration_hours),
    )
    .await
}

async fn send_to_first_backend_from_now(
    req: Request<Incoming>,
    config: Arc<config::Config>,
    direction: parameters::SearchDirection,
) -> Result<Response<Incoming>, ErrorBox> {
    let max_duration_hours = config.max_duration_hours;
    send_to_first_backend(
        req,
        config,
        get_offset_from_now(direction, max_duration_hours),
    )
    .await
}

async fn send_to_first_backend(
    req: Request<Incoming>,
    config: Arc<config::Config>,
    current_day_offset: i32,
) -> Result<Response<Incoming>, ErrorBox> {
    println!(
        "Forwarding request to '{}'. Days required on backend: {}",
        req.uri().path(),
        current_day_offset + 1
    );
    for backend in &config.backends {
        if backend.can_route_in_days(current_day_offset) {
            return forward_request(req, &backend.backend_address).await;
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
