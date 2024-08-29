use std::env;
use reqwest::blocking::ClientBuilder;
use reqwest::Proxy;

pub fn get_sync_client() -> ClientBuilder {
    let proxy_val  = env::var("PODFETCH_PROXY");



    let mut res = ClientBuilder::new();

    if let Ok(unwrapped_proxy) = proxy_val {
        let proxy = Proxy::all(unwrapped_proxy);
        match proxy {
            Ok(e)=>{
                res = res.proxy(e);
            }
            Err(e)=>{
                log::error!("Proxy is invalid {}", e)
            }
        }
    }

    res
}

pub fn get_async_sync_client() -> reqwest::ClientBuilder {
    let proxy_val  = env::var("PODFETCH_PROXY");

    let mut res = reqwest::ClientBuilder::new();

    if let Ok(unwrapped_proxy) = proxy_val {
        let proxy = Proxy::all(unwrapped_proxy);
        match proxy {
            Ok(e)=>{
                res = res.proxy(e);
            }
            Err(e)=>{
                log::error!("Proxy is invalid {}", e)
            }
        }
    }

    res
}