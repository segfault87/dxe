#![allow(clippy::too_many_arguments)]

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
use dxe_extern::toss_payments::TossPaymentsClient;
use jwt_compact::alg::Ed25519;

use crate::config::Config;
use crate::middleware::coordinator_verifier::PublicKeyBundle;
use crate::services::calendar::CalendarService;
use crate::services::doorlock::DoorLockService;
use crate::services::messaging::MessagingService;
use crate::services::telemetry::spawn_notification_service_task;
use crate::session::UserSession;
use crate::utils::aes::AesCrypto;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, default_value = "false")]
    migrate: bool,
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

    if args.migrate {
        let mut connection = database.acquire().await?;

        sqlx::migrate!("../data/migrations")
            .run(&mut connection)
            .await?;
        return Ok(());
    }

    let aes = AesCrypto::new(config.aes_key.as_slice());
    let aes = Data::new(aes);

    let booking_config = Data::new(config.booking.clone());
    let timezone_config = Data::new(config.timezone.clone());
    let doorlock_service = Data::new(DoorLockService::new(&config.spaces));
    let s2s_public_keys = Arc::new(PublicKeyBundle::new(&config.spaces));
    let calendar_service = if let Some(google_api_config) = &config.google_apis {
        Some(CalendarService::new(
            google_api_config,
            config.timezone.clone(),
        )?)
    } else {
        None
    };
    let toss_payments_client = Data::new(TossPaymentsClient::new(&config.toss_payments));

    let (messaging_service, messaging_consumers) = MessagingService::new(
        &config.messaging,
        config.timezone.clone(),
        config.url.clone(),
    );
    let messaging_service = Data::new(messaging_service);

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
            .app_data(doorlock_service.clone())
            .app_data(toss_payments_client.clone())
            .app_data(messaging_service.clone())
            .app_data(Data::new(notification_sender.clone()))
            .app_data(Data::new(config.url.clone()))
            .app_data(Data::new(calendar_service.clone()))
            .app_data(Data::new(config.telemetry.clone()))
            .service(controller::api(authority, s2s_public_keys.clone()))
    })
    .bind(&args.address)?
    .run()
    .await?;

    notification_task.abort();
    for messaging_consumer in messaging_consumers {
        messaging_consumer.abort();
    }

    Ok(())
}
