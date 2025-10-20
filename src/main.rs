use hyper::{
    Request, Response, body::Incoming, client::conn::http1::SendRequest, server::conn::http1,
};
use hyper_util::rt::TokioIo;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceBuilder;
use url::Url;

mod config;
mod path_rewriter;

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

fn parse_query(query: Option<&str>) -> Option<HashMap<String, String>> {
    query.and_then(|query| {
        let url = format!("http://localhost/?{}", query);
        Url::parse(url.as_str())
            .and_then(|parsed| Ok(parsed.query_pairs().into_owned().collect()))
            .ok()
    })
}

async fn proxy(
    req: Request<Incoming>,
    config: Arc<config::Config>,
) -> Result<Response<Incoming>, Infallible> {
    // println!("Request: {:?}", req);
    let query_parameters = parse_query(req.uri().query());
    // println!("Query Parameters: {:?}", query_parameters);
    forward_request(req, &config.backend_address).await
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
