use std::collections::HashMap;
use std::future::{Ready, ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_http::Method;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::web::BytesMut;
use actix_web::{FromRequest, HttpMessage};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use dxe_types::SpaceId;
use ed25519_compact::{PublicKey, Signature};
use futures::StreamExt;
use futures::future::LocalBoxFuture;

use crate::config::SpaceConfig;
use crate::models::Error;

#[derive(Clone, Debug)]
pub struct CoordinatorContext {
    pub space_id: SpaceId,
}

impl FromRequest for CoordinatorContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        ready(req.extensions().get().cloned().ok_or(Error::Forbidden))
    }
}

pub struct PublicKeyBundle {
    map: HashMap<SpaceId, ed25519_compact::PublicKey>,
}

impl PublicKeyBundle {
    pub fn new(config: &HashMap<SpaceId, SpaceConfig>) -> Self {
        let mut map = HashMap::new();
        for (space_id, config) in config.iter() {
            let public_key = PublicKey::from_slice(config.security.public_key.as_slice())
                .expect("32 bytes ed25519 public key");
            map.insert(space_id.clone(), public_key);
        }

        Self { map }
    }
}

pub struct CoordinatorVerifier {
    public_keys: Arc<PublicKeyBundle>,
}

impl CoordinatorVerifier {
    pub fn new(public_keys: Arc<PublicKeyBundle>) -> Self {
        Self { public_keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CoordinatorVerifier
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = CoordinatorVerifierMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CoordinatorVerifierMiddleware {
            public_keys: self.public_keys.clone(),
            service: Rc::new(service),
        }))
    }
}

pub struct CoordinatorVerifierMiddleware<S> {
    public_keys: Arc<PublicKeyBundle>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CoordinatorVerifierMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let public_keys = self.public_keys.clone();

        let service = self.service.clone();

        Box::pin(async move {
            let request_body = if req.method() == Method::POST || req.method() == Method::PUT {
                let mut request_body = BytesMut::new();
                while let Some(chunk) = req.take_payload().next().await {
                    request_body.extend_from_slice(&chunk.map_err(|_| Error::Forbidden)?);
                }

                let request_body = request_body.freeze();

                let (_, mut payload) = actix_http::h1::Payload::create(false);
                payload.unread_data(request_body.clone());
                req.set_payload(actix_http::Payload::H1 { payload });

                request_body
            } else {
                Default::default()
            };

            let headers = req.headers();

            let space_id = SpaceId::from(
                headers
                    .get("X-Space-Id")
                    .ok_or(Error::Forbidden)?
                    .to_str()
                    .map_err(|_| Error::Forbidden)?
                    .to_owned(),
            );
            let Some(public_key) = public_keys.map.get(&space_id) else {
                return Err(Error::Forbidden.into());
            };
            let signature = Signature::from_slice(
                BASE64_STANDARD
                    .decode(
                        headers
                            .get("X-Signature")
                            .ok_or(Error::Forbidden)?
                            .as_bytes(),
                    )
                    .map_err(|_| Error::Forbidden)?
                    .as_slice(),
            )
            .map_err(|_| Error::Forbidden)?;
            let expires_in_msecs = headers
                .get("X-Signature-Expires-In")
                .ok_or(Error::Forbidden)?
                .to_str()
                .map_err(|_| Error::Forbidden)?
                .parse::<i64>()
                .map_err(|_| Error::Forbidden)?;
            let expires_in =
                DateTime::from_timestamp_millis(expires_in_msecs).ok_or(Error::Forbidden)?;

            let now = Utc::now();

            if now > expires_in {
                return Err(Error::Forbidden.into());
            }

            let mut message = BytesMut::new();
            message.extend_from_slice(expires_in_msecs.to_string().as_bytes());
            message.extend_from_slice(req.method().as_str().as_bytes());
            message.extend_from_slice(req.path().as_bytes());
            message.extend_from_slice(req.query_string().as_bytes());
            message.extend(request_body);

            println!("message: {}", String::from_utf8_lossy(&message));

            public_key
                .verify(message.freeze(), &signature)
                .map_err(|_| Error::Forbidden)?;

            req.extensions_mut().insert(CoordinatorContext { space_id });

            service.call(req).await
        })
    }
}
