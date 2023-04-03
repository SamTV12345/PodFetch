use std::error::Error;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::Validation;
use reqwest::{Client, Response};

async fn fetch_jwks(jwks_uri: &str) -> Result<Jwk, reqwest::Error> {
    let client = Client::new();
    let response: Response = client
        .get(jwks_uri)
        .send()
        .await?;
    response.json::<Jwk>().await
}
