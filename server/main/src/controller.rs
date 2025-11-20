mod admin;
mod auth;
mod booking;
mod join_group;
mod user;

use actix_jwt_auth_middleware::{AuthenticationService, Authority};
use actix_state_guards::UseStateGuardOnScope;
use actix_web::{FromRequest, Handler, web};
use jwt_compact::Algorithm;
use serde::{Serialize, de::DeserializeOwned};

use crate::{models::Error, session::UserSession};

pub fn api<Claims, Algo, ReAuth, Args>(
    jwt: Authority<Claims, Algo, ReAuth, Args>,
) -> actix_web::Scope
where
    Algo: Algorithm + Clone + 'static,
    Algo::SigningKey: Clone,
    Args: FromRequest + 'static,
    Claims: DeserializeOwned + Serialize + 'static,
    ReAuth: Handler<Args, Output = Result<(), actix_web::Error>>,
{
    let scope_with_auth = web::scope("")
        .service(booking::bookings_scope())
        .service(booking::booking_scope())
        .service(user::scope())
        .use_state_guard(
            |session: UserSession| async move {
                if session.is_administrator {
                    Ok(())
                } else {
                    Err(Error::Forbidden)
                }
            },
            admin::scope(),
        )
        .wrap(AuthenticationService::new(jwt));

    web::scope("/api")
        .service(auth::scope())
        .service(join_group::resource())
        .service(scope_with_auth)
}
