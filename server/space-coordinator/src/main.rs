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
use crate::tasks::TaskContext;
use crate::tasks::booking_state_manager::BookingStateManager;
use crate::tasks::carpark_exempter::CarparkExempter;
use crate::tasks::mqtt_controller::MqttController;
use crate::tasks::presence_monitor::PresenceMonitor;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short)]
    config_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let config = toml::from_str::<Config>(&std::fs::read_to_string(&args.config_path)?)?;

    let mut client = DxeClient::new(
        config.space_id.clone(),
        config.url_base.clone(),
        config.request_expires_in(),
        config.private_key.as_slice(),
    )?;
    client.synchronize_clock().await?;

    let mut task_context = TaskContext::new().await?;

    let (presence_state, presence_monitor) = PresenceMonitor::new(&config.presence_monitor);
    let presence_monitor_task = presence_monitor.task();

    let (booking_states, mut booking_state_manager) =
        BookingStateManager::new(client.clone(), task_context.scheduler.clone());

    let (mqtt_service, mqtt_service_task) = MqttService::new(&config.mqtt);
    for device in config.z2m.devices.iter() {
        mqtt_service.subscribe(&device).await?;
    }

    let mqtt_controller =
        MqttController::new(&config.z2m, mqtt_service.clone(), presence_state.clone());
    let (mqtt_controller, mqtt_consumer, mqtt_controller_task) = mqtt_controller.task();
    booking_state_manager.add_callback(mqtt_controller);
    task_context.add_task(mqtt_controller_task).await?;

    let booking_state_manager_task = booking_state_manager.task();

    task_context.add_task(presence_monitor_task).await?;
    task_context.add_task(booking_state_manager_task).await?;

    if let Some(carpark_exemption) = &config.carpark_exemption {
        let carpark_exempter = CarparkExempter::new(
            booking_states.clone(),
            CarparkExemptionService::new(carpark_exemption),
        );

        task_context.add_task(carpark_exempter.task()).await?;
    }

    task_context.run().await;

    mqtt_service_task.abort();
    mqtt_consumer.abort();

    Ok(())
}
