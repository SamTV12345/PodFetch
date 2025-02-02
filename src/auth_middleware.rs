use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::user::User;
use base64::engine::general_purpose;
use base64::Engine;
use std::collections::HashSet;
use std::sync::LazyLock;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use crate::service::environment_service::ReverseProxyConfig;
use crate::utils::error::{CustomError, CustomErrorInner};
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::info;
use serde_json::Value;
use sha256::digest;
use crate::utils::reqwest_client::get_sync_client;

enum AuthType {
    Basic,
    Oidc,
    Proxy,
    None,
}

pub struct AuthFilter;


pub async fn handle_basic_auth(mut request: Request, next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Basic)?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

static JWKS: LazyLock<Jwk> = LazyLock::new(||{
   let sync_client = get_sync_client().build().unwrap();
    
    sync_client.get(&ENVIRONMENT_SERVICE.oidc_config.clone().unwrap().jwks_uri).send().unwrap().json::<Jwk>().unwrap()
});

pub async fn handle_oidc_auth(
    mut request: Request,
    next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Oidc)?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn handle_proxy_auth(
    mut request: Request,
    next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Proxy)?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn handle_no_auth(
    mut request: Request,
    next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::None)?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}


fn handle_auth_internal(req: &mut Request, auth_type: AuthType) -> Result<User, CustomError> {
    match auth_type {
        AuthType::Basic => AuthFilter::handle_basic_auth_internal(req),
        AuthType::Oidc => AuthFilter::handle_oidc_auth_internal(req),
        AuthType::Proxy => AuthFilter::handle_proxy_auth_internal(
            req,
            &ENVIRONMENT_SERVICE.reverse_proxy_config.clone().unwrap(),
        ),
        AuthType::None => Ok(User::create_standard_admin_user()),
    }
}


impl AuthFilter {
    pub fn extract_basic_auth(auth: &str) -> Result<(String, String), CustomError> {
        let auth = auth.to_string();
        let auth = auth.split(' ').collect::<Vec<&str>>();
        let auth = auth[1];
        let auth = general_purpose::STANDARD
            .decode(auth)
            .map_err(|_| CustomError::from(CustomErrorInner::Forbidden))?;
        let auth = String::from_utf8(auth).unwrap();
        let auth = auth.split(':').collect::<Vec<&str>>();
        let username = auth[0];
        let password = auth[1];
        Ok((username.to_string(), password.to_string()))
    }

    pub fn basic_auth_login(rq: &str) -> Result<(String, String), CustomError> {
        let (u, p) = Self::extract_basic_auth(rq)?;

        Ok((u.to_string(), p.to_string()))
    }

    fn handle_basic_auth_internal(req: &Request) -> Result<User, CustomError> {
        let opt_auth_header = req.headers().get("Authorization");
        match opt_auth_header {
            Some(header) => match header.to_str() {
                Ok(auth) => {
                    let (user, password) = AuthFilter::extract_basic_auth(auth)?;

                    let found_user =
                        User::find_by_username(&user).map_err(|_| CustomErrorInner::Forbidden)?;

                    if let Some(password_from_user) = &found_user.password {
                        if password_from_user == &digest(password) {
                            Ok(found_user)
                        } else {
                            Err(CustomErrorInner::Forbidden.into())
                        }
                    } else {
                        Err(CustomErrorInner::Forbidden.into())
                    }
                }
                Err(_) => Err(CustomErrorInner::Forbidden.into()),
            },
            None => Err(CustomErrorInner::Forbidden.into()),
        }
    }

    fn handle_oidc_auth_internal(req: &mut Request) ->
                                                                                     Result<User, CustomError> {
        let token_res = match req.headers().get("Authorization") {
            Some(token) => match token.to_str() {
                Ok(token) => Ok(token),
                Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden)),
            },
            None => Err(CustomErrorInner::UnAuthorized("Unauthorized".to_string()).into()),
        }?;

        let token = token_res.replace("Bearer ", "");
        let key = DecodingKey::from_jwk(&JWKS).unwrap();
        let mut validation = Validation::new(Algorithm::RS256);
        let mut aud_hashset = HashSet::new();
        aud_hashset.insert(ENVIRONMENT_SERVICE.oidc_config.clone().unwrap().client_id.to_string());
        validation.aud = Some(aud_hashset);

        let decoded_token = match decode::<Value>(&token, &key, &validation) {
            Ok(decoded) => Ok(decoded),
            Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden)),
        }?;
        let username = decoded_token
            .claims
            .get("preferred_username")
            .unwrap()
            .as_str()
            .unwrap();
        let found_user = User::find_by_username(username);

        match found_user {
            Ok(user) => Ok(user),
            Err(_) => {
                let preferred_username_claim = match decoded_token.claims.get("preferred_username")
                {
                    Some(claim) => match claim.as_str() {
                        Some(content) => Ok(content),
                        None => Err(CustomError::from(CustomErrorInner::Forbidden)),
                    },
                    None => Err(CustomErrorInner::Forbidden.into()),
                }?;

                // User is authenticated so we can onboard him if he is new
                let user = User {
                    id: 0,
                    username: preferred_username_claim.to_string(),
                    role: "user".to_string(),
                    password: None,
                    explicit_consent: false,
                    created_at: chrono::Utc::now().naive_utc(),
                    api_key: None,
                }
                .insert_user()?;
                Ok(user)
            }
        }
    }

    fn handle_proxy_auth_internal(
        req: &Request,
        reverse_proxy_config: &ReverseProxyConfig,
    ) -> Result<User, CustomError> {
        let header_val = match req
            .headers()
            .get(reverse_proxy_config.header_name.to_string())
        {
            Some(header) => Ok::<&HeaderValue, CustomError>(header),
            None => {
                info!("Reverse proxy is enabled but no header is provided");
                return Err(CustomError::from(CustomErrorInner::Forbidden));
            }
        }?;
        let token_res = match header_val.to_str() {
            Ok(token) => Ok(token),
            Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden)),
        }?;
        let found_user = User::find_by_username(token_res);

        match found_user {
            Ok(user) => Ok(user),
            Err(_) => {
                if reverse_proxy_config.auto_sign_up {
                    let user = User {
                        id: 0,
                        username: token_res.to_string(),
                        role: "user".to_string(),
                        password: None,
                        explicit_consent: false,
                        created_at: chrono::Utc::now().naive_utc(),
                        api_key: None,
                    }
                    .insert_user()
                    .expect("Error inserting user");
                    Ok(user)
                } else {
                    Err(CustomErrorInner::Forbidden.into())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use axum::extract::Request;
    
    
    use serial_test::serial;
    use crate::auth_middleware::AuthFilter;
    use crate::commands::startup::handle_config_for_server_startup;
    use crate::service::environment_service::ReverseProxyConfig;
    use crate::test_utils::test::{ create_random_user, ContainerCommands, POSTGRES_CHANNEL};

    #[test]
    #[serial]
    fn test_basic_auth_login() {
        let result = AuthFilter::extract_basic_auth("Bearer dGVzdDp0ZXN0");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test");
        assert_eq!(p, "test");
    }

    #[test]
    #[serial]
    fn test_basic_auth_login_with_special_characters() {
        let result = AuthFilter::extract_basic_auth("Bearer dGVzdCTDvMOWOnRlc3Q=");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test$üÖ");
        assert_eq!(p, "test");
    }

    #[serial]
    #[tokio::test]
    async fn test_proxy_auth_with_no_header_in_request_auto_sign_up() {
        let _ = handle_config_for_server_startup();
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = Request::builder().header("Content-Type", "text/plain").body("".into()).unwrap();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up() {
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();

        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: false,
        };
        let req = Request::builder().header("Content-Type", "text/plain")
            .header("X-Auth", "test")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up_false() {
        let _ = handle_config_for_server_startup();
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("X-Auth", "test")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_ok());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_no_header() {
        let _ = handle_config_for_server_startup();
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();

        let req = Request::builder().header("Content-Type", "text/plain")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_no_user() {
        let _ = handle_config_for_server_startup();
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_other_user() {
        // given
        let _ = handle_config_for_server_startup();
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();
        create_random_user().insert_user().unwrap();

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into()).unwrap();

        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_correct_user_wrong_password() {
        // given
        let _ = handle_config_for_server_startup();
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Cleanup).unwrap();
        create_random_user().insert_user().unwrap();

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdHVzZXI6dGVzdA==")
            .body("".into()).unwrap();
        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }
}
