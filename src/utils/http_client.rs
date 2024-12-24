use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use reqwest::{Client, Proxy};
use std::sync::{LazyLock, OnceLock};

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

pub static HTTP_CLIENT_WITH_HEADERS: LazyLock<Client> = LazyLock::new(Client::new);

pub fn get_http_client() -> Client {
    HTTP_CLIENT
        .get_or_init(|| get_async_sync_client().build().unwrap())
        .clone()
}

pub fn get_async_sync_client() -> reqwest::ClientBuilder {
    let mut res = reqwest::ClientBuilder::new();

    if let Some(unwrapped_proxy) = ENVIRONMENT_SERVICE.proxy_url.clone() {
        let proxy = Proxy::all(unwrapped_proxy);
        match proxy {
            Ok(e) => {
                res = res.proxy(e);
            }
            Err(e) => {
                log::error!("Proxy is invalid {}", e)
            }
        }
    }

    res
}
