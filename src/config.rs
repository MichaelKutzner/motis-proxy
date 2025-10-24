use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

type BackendAddress = String;

#[derive(Debug)]
pub struct Config {
    pub default_backend_address: BackendAddress,
    pub backends: Vec<Backend>,
    pub bind_addr: (IpAddr, u16),
    pub subpath: String,
    pub max_duration_hours: i32,
}

impl Config {
    pub fn load() -> Arc<Self> {
        Arc::new(Self {
            default_backend_address: parse_default_backend(
                option_env!("BACKEND_ADDRESS"),
                "http://127.0.0.1:8080",
            ),
            backends: parse_backends(option_env!("BACKENDS")),
            bind_addr: parse_bind_address(option_env!("BIND_ADDR"), option_env!("BIND_PORT")),
            subpath: parse_prefix(option_env!("PROXY_PREFIX")),
            max_duration_hours: parse_max_duration_hours(option_env!("MAX_HOURS")),
        })
    }
}

fn parse_default_backend(backend: Option<&str>, default: &str) -> BackendAddress {
    let backend_uri = backend.unwrap_or(default);
    let backend_uri = backend_uri
        .parse::<hyper::Uri>()
        .expect("Failed to parse BACKEND_URI");
    let host = backend_uri.host().expect("Missing backend host");
    let port = backend_uri.port_u16().unwrap_or(80);
    format!("{}:{}", host, port)
}

fn parse_backends(backends: Option<&str>) -> Vec<Backend> {
    backends
        .and_then(|backends| {
            Some(
                backends
                    .split(';')
                    .into_iter()
                    .map(Backend::new)
                    .collect::<Vec<Backend>>(),
            )
        })
        .unwrap_or_default()
}

fn parse_bind_address(bind_addr: Option<&str>, bind_port: Option<&str>) -> (IpAddr, u16) {
    let host = bind_addr
        .and_then(|ip| ip.parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    let port = bind_port
        .and_then(|ip| ip.parse::<u16>().ok())
        .unwrap_or(5173);
    (host, port)
}

fn parse_prefix(prefix: Option<&str>) -> String {
    prefix.unwrap_or("").to_string()
}

fn parse_max_duration_hours(max_duration_hours: Option<&str>) -> i32 {
    max_duration_hours
        .and_then(|max_hours| max_hours.parse::<i32>().ok())
        .unwrap_or(12)
}

#[derive(Debug)]
pub struct Backend {
    days: i32,
    pub backend_address: BackendAddress,
}

impl Backend {
    pub fn can_route_in_days(&self, days: i32) -> bool {
        self.days > days
    }
    fn new(days_backend: &str) -> Self {
        let (days, backend) = days_backend
            .split_once('#')
            .expect(format!("Invalid backend format: '{}'", days_backend).as_str());
        Self {
            days: days.parse::<i32>().expect("Invalid number of days"),
            backend_address: backend.to_string(),
        }
    }
}
