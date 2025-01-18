use std::future::Future;
use std::pin::Pin;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::models::session::Session;
use axum_extra::extract::cookie::CookieJar;

use diesel::ExpressionMethods;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use std::task::{Context, Poll};
use axum::handler::Handler;
use axum::http;
use axum::{
    response::Response,
    extract::Request,
};
use tower::{Layer, MakeService, Service};
use crate::utils::error::{CustomErrorInner};

pub struct CookieFilter {}

impl<S> Layer<S> for CookieFilter {
    type Service = MyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware { inner }
    }
}

#[derive(Clone)]
struct MyMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Vec<u8>>> for MyMiddleware<S> {
    type Response = Response<Vec<u8>>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Vec<u8>>) -> Self::Future {
        let jar = CookieJar::from_headers(req.headers());
        let cookie = jar.get("sessionid");
        if cookie.is_none() {
            return Box::pin(Err(CustomErrorInner::Forbidden.into()));
        }
        let binding = cookie.unwrap();
        let extracted_cookie = binding.value();

        use crate::adapters::persistence::dbconfig::schema::sessions::dsl::*;
        let session = sessions
            .filter(session_id.eq(extracted_cookie))
            .first::<Session>(&mut get_connection())
            .optional()
            .expect("Error connecting to database");
        if session.is_none() {
            return Box::pin(Err(CustomErrorInner::Forbidden.into()));
        }

        req.extensions_mut().insert(session.unwrap());
        let future = self.inner.call(req);


        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
