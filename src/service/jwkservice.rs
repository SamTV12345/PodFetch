use crate::models::oidc_model::CustomJwkSet;

#[derive(Clone)]
pub struct JWKService {
    pub jwk: Option<CustomJwkSet>,
    pub timestamp: u64
}