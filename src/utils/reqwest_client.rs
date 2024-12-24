use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use reqwest::blocking::ClientBuilder;
use reqwest::Proxy;

pub fn get_sync_client() -> ClientBuilder {
    let mut res = ClientBuilder::new();

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
