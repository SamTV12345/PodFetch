use crate::models::oidc_model::CustomJwkSet;

#[derive(Clone)]
pub struct JWKService {
    pub jwk: Option<CustomJwkSet>,
    pub timestamp: u64,
}

impl JWKService {
    pub fn new() -> Self {
        JWKService {
            jwk: None,
            timestamp: 0,
        }
    }
}
