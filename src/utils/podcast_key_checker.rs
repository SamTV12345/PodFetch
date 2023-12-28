use std::collections::HashMap;
use actix_web::{Error, HttpRequest};
use actix_web::dev::{Service, ServiceRequest};
use actix_web::error::ErrorUnauthorized;
use futures_util::FutureExt;
use crate::config::dbconfig::establish_connection;
use crate::constants::inner_constants::{BASIC_AUTH, OIDC_AUTH};
use crate::models::user::User;
use crate::utils::environment_variables::is_env_var_present_and_true;

pub fn check_podcast_request<S, B>(
    req: ServiceRequest,
    srv: &S,
) -> impl futures::Future<Output = Result<S::Response, Error>>
    where
        S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
{
    let is_auth_enabled = is_env_var_present_and_true(BASIC_AUTH) || is_env_var_present_and_true(OIDC_AUTH);

    if is_auth_enabled {
        let mut hash = HashMap::new();
        let query = req.query_string();

        if query.trim().len() == 0 {
            return async {Err(ErrorUnauthorized("Unauthorized"))}.boxed_local()
        }

        query.split("&").for_each(|v| {
            let mut split = v.split("=");
            hash.insert(split.next().unwrap(), split.next().unwrap());
        });
        let api_key = hash.get("apiKey");

        if api_key.is_none() {
            return async {Err(ErrorUnauthorized("Unauthorized"))}.boxed_local()
        }

        let conn = &mut establish_connection();
        let api_key_exists = User::check_if_api_key_exists(api_key.unwrap().to_string(), conn);
        if !api_key_exists {
            return async {Err(ErrorUnauthorized("Unauthorized"))}.boxed_local();
        }
    }
    Box::pin(srv.call(req))
}