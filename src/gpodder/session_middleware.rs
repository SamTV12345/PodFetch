use crate::config::dbconfig::establish_connection;
use crate::models::session::Session;
use actix::fut::ok;
use actix_web::body::{EitherBody, MessageBody};
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use diesel::ExpressionMethods;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use futures_util::future::{LocalBoxFuture, Ready};
use futures_util::FutureExt;
use std::rc::Rc;

pub struct CookieFilter {}

impl CookieFilter {
    pub fn new() -> Self {
        CookieFilter {}
    }
}

pub struct CookieFilterMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Transform<S, ServiceRequest> for CookieFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = CookieFilterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CookieFilterMiddleware {
            service: Rc::new(service),
        })
    }
}

impl<S, B> Service<ServiceRequest> for CookieFilterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let cookie = req.cookie("sessionid");
        if cookie.is_none() {
            return Box::pin(ok(req
                .error_response(ErrorUnauthorized("Unauthorized"))
                .map_into_right_body()));
        }
        let binding = cookie.unwrap();
        let extracted_cookie = binding.value();

        use crate::dbconfig::schema::sessions::dsl::*;
        let session = sessions
            .filter(session_id.eq(extracted_cookie))
            .first::<Session>(&mut establish_connection())
            .optional()
            .expect("Error connecting to database");
        if session.is_none() {
            return Box::pin(ok(req
                .error_response(ErrorForbidden("Forbidden"))
                .map_into_right_body()));
        }

        let service = Rc::clone(&self.service);

        req.extensions_mut().insert(session.unwrap());
        async move { service.call(req).await.map(|res| res.map_into_left_body()) }.boxed_local()
    }
}
