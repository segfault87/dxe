mod config;
mod controller;
mod middleware;
mod models;
mod services;
mod session;
mod utils;

use std::sync::Arc;
use std::time::Duration;

use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_web::web::Data;
use clap::Parser;
use jwt_compact::alg::Ed25519;

use crate::config::Config;
use crate::middleware::coordinator_verifier::PublicKeyBundle;
use crate::services::doorlock::DoorLockService;
use crate::services::messaging::biztalk::BiztalkClient;
use crate::services::messaging::spawn_messaging_backend;
use crate::services::telemetry::spawn_notification_service_task;
use crate::session::UserSession;
use crate::utils::aes::AesCrypto;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short)]
    config_path: std::path::PathBuf,
    #[arg(default_value = "127.0.0.1:8000")]
    address: std::net::SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let mut config = toml::from_str::<Config>(&std::fs::read_to_string(&args.config_path)?)?;
    config.booking.sanitize();

    let database = sqlx::SqlitePool::connect(config.database.url.as_str()).await?;
    let database = Data::new(database);

    let aes = AesCrypto::new(config.aes_key.as_slice());
    let aes = Data::new(aes);

    let booking_config = Data::new(config.booking.clone());

    let timezone_config = Data::new(config.timezone.clone());

    let doorlock_service = DoorLockService::new(&config.spaces);

    let s2s_public_keys = Arc::new(PublicKeyBundle::new(&config.spaces));

    let (biztalk_task, biztalk_sender) = if let Some(config) = &config.messaging.biztalk {
        let backend = BiztalkClient::new(config);
        let (task, sender) = spawn_messaging_backend(backend);
        (Some(task), Some(sender))
    } else {
        (None, None)
    };

    let (notification_task, notification_sender) =
        spawn_notification_service_task(config.notifications.clone());

    let key_pair = config.jwt.key_pair()?;

    actix_web::HttpServer::new(move || {
        let authority = Authority::<UserSession, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(key_pair.sk.clone())
                    .algorithm(Ed25519)
                    .refresh_token_lifetime(Duration::from_secs(60 * 60 * 24 * 30))
                    .build()
                    .unwrap(),
            ))
            .verifying_key(key_pair.pk)
            .build()
            .unwrap();

        actix_web::App::new()
            .app_data(Data::new(config.auth.kakao.clone()))
            .app_data(Data::new(authority.token_signer().unwrap()))
            .app_data(database.clone())
            .app_data(aes.clone())
            .app_data(timezone_config.clone())
            .app_data(booking_config.clone())
            .app_data(Data::new(notification_sender.clone()))
            .app_data(Data::new(doorlock_service.clone()))
            .app_data(Data::new(biztalk_sender.clone()))
            .app_data(Data::new(config.url.clone()))
            .service(controller::api(authority, s2s_public_keys.clone()))
    })
    .bind(&args.address)?
    .run()
    .await?;

    notification_task.abort();
    if let Some(v) = biztalk_task {
        v.abort()
    }

    Ok(())
}
