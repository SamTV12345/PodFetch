use jwt_simple::claims::{Claims, JWTClaims};
use jwt_simple::prelude::{Duration, NoCustomClaims, RSAKeyPairLike, RSAPublicKeyLike};
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::dbconfig::DBType;
use crate::models::settings::Setting;
use crate::utils::error::CustomError;

#[derive(Serialize, Deserialize)]
pub struct WatchTogetherUser {
    pub preferred_username: String,
}


pub fn generate_watch_together_id(username: Option<String>, opt_subject: Option<String>, conn: &mut
DBType)
                                  -> String {
    //unwrap: Safe as the key is generated at startup
    let watch_together_claims = WatchTogetherUser{
        preferred_username: username.unwrap_or("Unknown user".into())
    };
    let server_url = &ENVIRONMENT_SERVICE.get().unwrap().server_url;
    let jwt_key = Setting::get_jwt_key(conn).unwrap();
    let claims = Claims::with_custom_claims(watch_together_claims, Duration::from_days(30))
        .with_issuer(server_url)
        .with_subject(opt_subject.unwrap_or(generate_subject()))
        .with_audience("watch_together");
    jwt_key.sign(claims).unwrap()
}


pub fn decode_watch_together_id(token: &str, conn: &mut DBType) -> Result<JWTClaims<NoCustomClaims>, CustomError> {
    //unwrap: Safe as the key is generated at startup
    let jwt_key = Setting::get_jwt_key(conn)?;
    jwt_key.public_key().verify_token(token, None).map_err(|e| CustomError::BadRequest
        (format!("Invalid token with reason: {e}")))
}

fn generate_subject() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}