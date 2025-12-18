use std::convert::Infallible;
use std::ops::Deref;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{FromRequest, HttpMessage};
use chrono::{DateTime, Utc};
use futures::future::{LocalBoxFuture, Ready, ready};

#[derive(Copy, Clone, Debug)]
pub struct Now(DateTime<Utc>);

impl Now {
    fn new() -> Self {
        Self(Utc::now())
    }
}

impl Deref for Now {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Now {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        let extensions = req.extensions();
        let now = extensions.get::<Now>();

        ready(Ok(*now.unwrap()))
    }
}

pub struct DateTimeInjector;

impl<S, B> Transform<S, ServiceRequest> for DateTimeInjector
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = DateTimeInjectorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DateTimeInjectorMiddleware { service }))
    }
}

pub struct DateTimeInjectorMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for DateTimeInjectorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        req.extensions_mut().insert(Now::new());

        Box::pin(self.service.call(req))
    }
}
