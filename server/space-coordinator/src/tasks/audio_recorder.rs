use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::sync::{Arc, Mutex};

use chrono::{TimeDelta, Utc};
use dxe_extern::google_cloud::drive::{Error as DriveError, GoogleDriveClient, SCOPE};
use dxe_extern::google_cloud::{Error as GcpError, get_token};
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::UpdateAudioRequest;
use dxe_types::{BookingId, UnitId};
use tokio::process;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::client::DxeClient;
use crate::config::{AudioRecorderConfig, GoogleApiConfig};

pub struct AudioRecorder {
    drive_client: GoogleDriveClient,
    dxe_client: DxeClient,

    config: HashMap<UnitId, AudioRecorderConfig>,

    tasks: Mutex<HashMap<BookingId, process::Child>>,
}

impl AudioRecorder {
    pub async fn new(
        google_api_config: &GoogleApiConfig,
        config: HashMap<UnitId, AudioRecorderConfig>,
        dxe_client: DxeClient,
    ) -> Result<Self, Error> {
        let token = get_token(google_api_config, &[SCOPE]).await?;
        let drive_client = GoogleDriveClient::new(token, &google_api_config.drive);

        Ok(Self {
            drive_client,
            dxe_client,

            config,

            tasks: Mutex::new(HashMap::new()),
        })
    }

    fn start(&self, config: &AudioRecorderConfig, booking: &BookingWithUsers) -> Result<(), Error> {
        let mut path = config.path_prefix.clone();
        path.push(format!("{}.mp3", booking.booking.id));

        let mut command = process::Command::new("sh");
        command.arg("-c").arg(format!(
            "{} --rate {} --target={} - | {} -s {} -r -b {} - -o {}",
            config.pw_record_bin,
            config.sampling_rate,
            config.target_device,
            config.lame_bin,
            config.sampling_rate as f32 / 1000.0,
            config.mp3_bitrate,
            path.to_str().ok_or(Error::InvalidPath)?,
        ));

        let child = command.spawn()?;

        self.tasks.lock().unwrap().insert(booking.booking.id, child);

        log::info!("Recording started for {}", booking.booking.id);

        Ok(())
    }

    async fn stop(&self, config: &AudioRecorderConfig, booking: &BookingWithUsers) {
        let Some(mut child) = self.tasks.lock().unwrap().remove(&booking.booking.id) else {
            return;
        };

        let _ = child.kill().await;

        log::info!(
            "Recording finished for {}. Uploading to Google Drive...",
            booking.booking.id
        );

        let mut path = config.path_prefix.clone();
        path.push(format!("{}.mp3", booking.booking.id));
        if !path.exists() {
            log::warn!("File {path:?} not exists.");
            return;
        }

        let drive_client = self.drive_client.clone();
        let dxe_client = self.dxe_client.clone();
        let booking_id = booking.booking.id.clone();
        tokio::task::spawn(async move {
            let url = match drive_client.upload(path, "audio/mp3").await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Couldn't upload file to Google Drive: {e}");
                    return;
                }
            };

            let expires_in = Utc::now() + TimeDelta::days(7);

            match dxe_client
                .post::<_, serde_json::Value>(
                    &format!("/booking/{}/audio", booking_id),
                    None,
                    UpdateAudioRequest {
                        url,
                        expires_in: expires_in.into(),
                    },
                )
                .await
            {
                Ok(_) => log::info!("Audio for {booking_id} updated successfully"),
                Err(e) => log::error!("Couldn't post audio information to server: {e}"),
            }
        });
    }

    async fn update(self: Arc<Self>) {
        let mut tasks = self.tasks.lock().unwrap();

        let mut tasks_to_collect = HashSet::new();
        for (key, task) in tasks.iter() {
            if task.id().is_none() {
                log::error!("Audio recording process for {key} exited prematurely.");
                tasks_to_collect.insert(key.clone());
            }
        }

        tasks.retain(|k, _| !tasks_to_collect.contains(k));
    }

    pub fn task(self) -> (Arc<Self>, Task) {
        let arc_self = Arc::new(self);

        (
            arc_self.clone(),
            TaskBuilder::new("audio_recorder", move || {
                let arc_self = arc_self.clone();
                tokio::task::spawn(async move {
                    arc_self.update().await;
                });

                Ok(())
            })
            .every_seconds(10)
            .build(),
        )
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for AudioRecorder {
    async fn on_event_created(
        &self,
        event: &BookingWithUsers,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if is_in_progress {
            if let Some(config) = self.config.get(&event.booking.unit_id) {
                if let Err(e) = self.start(config, event) {
                    log::warn!("Could not create audio recorder task: {e}");
                }
            }
        }

        Ok(())
    }

    async fn on_event_deleted(
        &self,
        event: &BookingWithUsers,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if is_in_progress {
            if let Some(config) = self.config.get(&event.booking.unit_id) {
                self.stop(config, event).await;
            }
        }

        Ok(())
    }

    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            if let Some(config) = self.config.get(&event.booking.unit_id) {
                if let Err(e) = self.start(config, event) {
                    log::warn!("Could not create audio recorder task: {e}");
                }
            }
        }

        Ok(())
    }

    async fn on_event_end(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            if let Some(config) = self.config.get(&event.booking.unit_id) {
                self.stop(config, event).await;
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not obtain GCP credential: {0}")]
    GcpAuth(#[from] GcpError),
    #[error("Could not upload to Google Drive: {0}")]
    Drive(#[from] DriveError),
    #[error("Invalid path")]
    InvalidPath,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
