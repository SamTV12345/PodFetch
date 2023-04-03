use std::fmt;
use std::fmt::{Formatter, write};

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwk {
    kid: String,
    kty: String,
    alg: String,
    use_: String,
    n: String,
    e: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomJwkSet {
    pub(crate) keys: Vec<CustomJwk>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomJwk{
    pub kid: String,
    pub kty: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub n: String,
    pub e: String,
}

impl fmt::Display for CustomJwk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write(f, format_args!("kid: {}, kty: {}, alg: {}, use_: {}, n: {}, e: {}", self.kid,
                              self.kty,
              self.alg, self.use_, self.n, self.e))
    }
}