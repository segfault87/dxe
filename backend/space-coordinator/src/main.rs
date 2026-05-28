mod callback;
mod client;
mod config;
mod device;
mod events;
mod services;
mod tables;
mod tasks;
mod types;
mod utils;

use clap::Parser;

use crate::client::DxeClient;
use crate::config::Config;
use crate::events::EventSender;
use crate::services::carpark_exemption::CarparkExemptionService;
use crate::services::mqtt::MqttService;
use crate::services::notification::NotificationService;
use crate::services::table_manager::TableManager;
use crate::tasks::TaskContext;
use crate::tasks::action_controller::{ActionController, DeviceController};
use crate::tasks::alert_publisher::AlertPublisher;
use crate::tasks::audio_recorder::AudioRecorder;
use crate::tasks::booking_reminder::BookingReminder;
use crate::tasks::booking_state_manager::BookingStateManager;
use crate::tasks::carpark_exempter::CarparkExempter;
use crate::tasks::metrics_publisher::MetricsPublisher;
use crate::tasks::notification_publisher::NotificationPublisher;
use crate::tasks::osd_controller::OsdController;
use crate::tasks::presence_monitor::PresenceMonitor;
use crate::tasks::sound_meter_controller::SoundMeterController;
use crate::tasks::telemetry_manager::TelemetryManager;
use crate::tasks::unit_fetcher::UnitFetcher;
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
    let config =
        toml::from_str::<Config>(&std::fs::read_to_string(&args.config_path)?)?.validate()?;

    let mut client = DxeClient::new(
        config.space_id.clone(),
        config.url_base.clone(),
        config.request_expires_in,
        config.private_key.as_slice(),
    )?;
    client.synchronize_clock().await?;

    let unit_fetcher = UnitFetcher::new(client.clone()).await?;
    let (unit_fetcher, unit_fetcher_task) = unit_fetcher.task();

    let notification_service = NotificationService::new(&config.notifications);

    let mut task_context = TaskContext::new().await?;

    let event_sender = EventSender::new();

    let booking_state_manager = BookingStateManager::new(
        &config.events.bookings,
        event_sender.clone(),
        client.clone(),
        task_context.scheduler.clone(),
    );

    let presence_monitor =
        PresenceMonitor::new(&config.presence_monitor, event_sender.clone()).await;
    let alert_publisher = AlertPublisher::new(&config.events.alerts, event_sender.clone());

    let action_controller = ActionController::new(config.triggers.clone());

    let (mqtt_service, mqtt_service_task) = MqttService::new(&config.mqtt);

    let mut z2m_controller = Z2mController::new(&config.z2m, mqtt_service.clone());
    z2m_controller.start().await;

    let (sound_meter_controller, sound_meter_tasks) =
        SoundMeterController::new(config.sound_meters.iter())?;

    let mut metrics_publisher = MetricsPublisher::new(config.metrics.iter());

    let table_manager = TableManager::new(
        z2m_controller.publisher(),
        sound_meter_controller.publisher(),
        metrics_publisher.publisher(),
        presence_monitor.publisher(),
        booking_state_manager.publisher(),
    );

    let booking_reminder = BookingReminder::new(client.clone());

    let osd_controller = OsdController::new(client.clone(), mqtt_service.clone(), &config.osd);

    let telemetry_manager =
        TelemetryManager::new(&config.telemetry, client.clone(), table_manager.clone())?;

    let (z2m_controller, z2m_consumer_task, z2m_controller_task) = z2m_controller.task();
    let mut device_controller =
        DeviceController::new(booking_reminder.clone(), z2m_controller.clone());

    metrics_publisher.start(table_manager.clone());

    let audio_recorder = AudioRecorder::new(
        &config.google_apis,
        config.audio_recorder.clone(),
        client.clone(),
        sound_meter_controller.publisher(),
    )
    .await?;
    let (audio_recorder, audio_recorder_task) = audio_recorder.task();

    let (osd_controller, osd_message_handler_task, osd_task) =
        osd_controller.start(event_sender.clone()).await?;

    let _alert_publisher = alert_publisher.start(table_manager.clone());

    let _notification_publisher = NotificationPublisher::new(
        notification_service.clone(),
        config.notifications.alerts.iter(),
    )
    .start(event_sender.clone());

    device_controller.add_booking_state_callback(audio_recorder);
    device_controller.add_booking_state_callback(telemetry_manager.clone());
    device_controller.add_osd_state_callback(osd_controller.clone());

    let device_controller = device_controller.build();

    action_controller
        .start(
            device_controller,
            event_sender.clone(),
            table_manager.clone(),
        )
        .await;

    if let Some(carpark_exemption) = &config.carpark_exemption {
        let carpark_exempter = CarparkExempter::new(
            client.clone(),
            CarparkExemptionService::new(carpark_exemption),
            osd_controller.clone(),
            notification_service.clone(),
            unit_fetcher.state(),
            booking_state_manager.get_states(),
        );

        let (_carpark_exempter, task) = carpark_exempter.task();

        task_context.add_task(task).await?;
    }

    let presence_monitor_task = presence_monitor.task();
    let booking_state_manager_task = booking_state_manager.task();

    task_context.add_task(unit_fetcher_task).await?;
    task_context.add_task(presence_monitor_task).await?;
    task_context.add_task(z2m_controller_task).await?;
    task_context.add_task(audio_recorder_task).await?;
    task_context.add_task(osd_task).await?;
    task_context.add_task(booking_state_manager_task).await?;

    task_context.run().await;

    telemetry_manager.abort();

    for sound_meter_task in sound_meter_tasks {
        sound_meter_task.abort();
    }
    z2m_consumer_task.abort();
    osd_message_handler_task.abort();
    mqtt_service_task.abort();

    Ok(())
}
