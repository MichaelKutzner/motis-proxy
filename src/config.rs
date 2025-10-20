use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub backend_address: String,
    pub bind_addr: (IpAddr, u16),
    pub subpath: String,
}

impl Config {
    pub fn load() -> Arc<Self> {
        let backend_uri = option_env!("BACKEND_URI").unwrap_or("http://127.0.0.1:8080");
        let backend_uri = backend_uri
            .parse::<hyper::Uri>()
            .expect("Failed to parse BACKEND_URI");
        let host = backend_uri.host().expect("Missing backend host");
        let port = backend_uri.port_u16().unwrap_or(80);
        let backend_address = format!("{}:{}", host, port);
        let host = option_env!("BIND_ADDR")
            .and_then(parse_ip)
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        let port = option_env!("BIND_PORT")
            .and_then(parse_port)
            .unwrap_or(5173);
        Arc::new(Self {
            backend_address: backend_address,
            bind_addr: (host, port),
            subpath: option_env!("PROXY_SUBPATH").unwrap_or("").to_string(),
        })
    }
}

fn parse_ip(ip: &str) -> Option<IpAddr> {
    ip.parse().ok()
}

fn parse_port(port: &str) -> Option<u16> {
    port.parse::<u16>().ok()
}
