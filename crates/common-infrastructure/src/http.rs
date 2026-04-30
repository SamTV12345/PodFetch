use crate::config::EnvironmentService;
use reqwest::blocking::ClientBuilder as BlockingClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder, Proxy};
use std::sync::OnceLock;

pub const COMMON_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36";

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_http_client(environment: &EnvironmentService) -> Client {
    HTTP_CLIENT
        .get_or_init(|| get_async_sync_client(environment).build().unwrap())
        .clone()
}

pub fn get_async_sync_client(environment: &EnvironmentService) -> ClientBuilder {
    let mut builder = ClientBuilder::new();
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "User-Agent",
        HeaderValue::from_str(COMMON_USER_AGENT).unwrap(),
    );

    if let Some(unwrapped_proxy) = environment.proxy_url.clone() {
        match Proxy::all(unwrapped_proxy) {
            Ok(proxy) => {
                builder = builder.proxy(proxy);
            }
            Err(error) => {
                tracing::error!("Proxy is invalid {error}");
            }
        }
    }

    builder.default_headers(header_map)
}

pub fn get_sync_client(environment: &EnvironmentService) -> BlockingClientBuilder {
    let mut builder = BlockingClientBuilder::new();
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "User-Agent",
        HeaderValue::from_str(COMMON_USER_AGENT).unwrap(),
    );

    if let Some(unwrapped_proxy) = environment.proxy_url.clone() {
        match Proxy::all(unwrapped_proxy) {
            Ok(proxy) => {
                builder = builder.proxy(proxy);
            }
            Err(error) => {
                tracing::error!("Proxy is invalid {error}");
            }
        }
    }

    builder.default_headers(header_map)
}
