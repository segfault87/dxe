mod callback;
mod client;
mod config;
mod services;
mod tasks;

use clap::Parser;

use crate::client::DxeClient;
use crate::config::Config;
use crate::services::carpark_exemption::CarparkExemptionService;
use crate::services::mqtt::MqttService;
use crate::services::notification::NotificationService;
use crate::tasks::TaskContext;
use crate::tasks::audio_recorder::AudioRecorder;
use crate::tasks::booking_state_manager::BookingStateManager;
use crate::tasks::carpark_exempter::CarparkExempter;
use crate::tasks::presence_monitor::PresenceMonitor;
use crate::tasks::telemetry_manager::TelemetryManager;
use crate::tasks::z2m_controller::Z2mController;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short)]
    config_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args = Args::parse();
    let config = toml::from_str::<Config>(&std::fs::read_to_string(&args.config_path)?)?;

    let mut client = DxeClient::new(
        config.space_id.clone(),
        config.url_base.clone(),
        config.request_expires_in(),
        config.private_key.as_slice(),
    )?;
    client.synchronize_clock().await?;

    let notification_service = NotificationService::new(&config.notifications);

    let mut task_context = TaskContext::new().await?;

    let (_presence_state, mut presence_monitor) = PresenceMonitor::new(&config.presence_monitor);

    let (booking_states, mut booking_state_manager) =
        BookingStateManager::new(client.clone(), task_context.scheduler.clone());

    let (mqtt_service, mqtt_service_task) = MqttService::new(&config.mqtt);

    let mut z2m_controller = Z2mController::new(
        &config.z2m,
        mqtt_service.clone(),
        notification_service.clone(),
    );
    z2m_controller.start().await;

    let telemetry_manager = TelemetryManager::new(&config.telemetry, client.clone());
    let telemetry_tasks = telemetry_manager
        .clone()
        .register_tables_from_config(&mut z2m_controller, &config.telemetry.tables)?;

    let (z2m_controller, z2m_consumer_task, z2m_controller_task) = z2m_controller.task();

    let audio_recorder = AudioRecorder::new(
        &config.google_apis,
        config.audio_recorder.clone(),
        client.clone(),
    )
    .await?;
    let (audio_recorder, audio_recorder_task) = audio_recorder.task();

    booking_state_manager.add_callback(z2m_controller.clone());
    booking_state_manager.add_callback(audio_recorder);
    booking_state_manager.add_callback(telemetry_manager.clone());

    presence_monitor.add_callback(z2m_controller);

    if let Some(carpark_exemption) = &config.carpark_exemption {
        let carpark_exempter = CarparkExempter::new(
            client.clone(),
            booking_states.clone(),
            CarparkExemptionService::new(carpark_exemption),
            notification_service.clone(),
        );

        let (carpark_exempter, task) = carpark_exempter.task();

        booking_state_manager.add_callback(carpark_exempter);
        task_context.add_task(task).await?;
    }

    let presence_monitor_task = presence_monitor.task();
    let booking_state_manager_task = booking_state_manager.task();

    task_context.add_task(presence_monitor_task).await?;
    task_context.add_task(z2m_controller_task).await?;
    task_context.add_task(audio_recorder_task).await?;

    task_context.add_task(booking_state_manager_task).await?;

    task_context.run().await;

    for task in telemetry_tasks {
        task.abort();
    }

    telemetry_manager.abort();

    z2m_consumer_task.abort();
    mqtt_service_task.abort();

    Ok(())
}
