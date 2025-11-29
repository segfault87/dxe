use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use chrono::{TimeDelta, Utc};
use dxe_extern::google_cloud::drive::{Error as DriveError, GoogleDriveClient};
use dxe_extern::google_cloud::{CredentialManager, Error as GcpError};
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::UpdateAudioRequest;
use dxe_types::{BookingId, UnitId};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::client::DxeClient;
use crate::config::{AudioRecorderConfig, GoogleApiConfig};

struct RecorderProcess {
    recorder: process::Child,
    lame: process::Child,
    stream_feeder: Option<tokio::task::JoinHandle<()>>,
    file_writer: Option<tokio::task::JoinHandle<()>>,
    has_stopped: Arc<Mutex<bool>>,
}

impl RecorderProcess {
    pub async fn new(config: &AudioRecorderConfig, output_path: PathBuf) -> Result<Self, Error> {
        let mut recorder_cmd = process::Command::new(config.pw_record_bin.clone());
        recorder_cmd
            .arg(format!("--rate={}", config.sampling_rate))
            .arg(format!("--target={}", config.target_device))
            .arg("-")
            .stdout(Stdio::piped());

        let mut lame_cmd = process::Command::new(config.lame_bin.clone());
        lame_cmd
            .arg("-r")
            .arg("-b")
            .arg(config.mp3_bitrate.to_string())
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut recorder = recorder_cmd.spawn()?;
        let mut recorder_stdout = recorder.stdout.take().unwrap();

        let mut lame = lame_cmd.spawn()?;
        let mut lame_stdout = lame.stdout.take().unwrap();
        let mut lame_stdin = lame.stdin.take().unwrap();

        let mut output_file = File::create(output_path).await?;

        let has_stopped = Arc::new(Mutex::new(false));

        let inner_has_stopped = has_stopped.clone();
        let stream_feeder = tokio::task::spawn(async move {
            let mut buffer = [0u8; 65536];

            while let Ok(read) = recorder_stdout.read(&mut buffer).await {
                if read == 0 {
                    break;
                }

                if let Err(e) = lame_stdin.write(&buffer[0..read]).await {
                    log::warn!("IO error while writing to file: {e}");
                    break;
                }
            }

            *inner_has_stopped.lock().unwrap() = true;
        });

        let file_writer = tokio::task::spawn(async move {
            let mut buffer = [0u8; 65536];

            while let Ok(read) = lame_stdout.read(&mut buffer).await {
                if read == 0 {
                    break;
                }

                if let Err(e) = output_file.write(&buffer[0..read]).await {
                    log::warn!("IO error while writing to file: {e}");
                    break;
                }
            }
        });

        Ok(RecorderProcess {
            recorder,
            lame,
            stream_feeder: Some(stream_feeder),
            file_writer: Some(file_writer),
            has_stopped,
        })
    }

    pub fn has_stopped(&self) -> bool {
        *self.has_stopped.lock().unwrap()
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        let Some(stream_feeder) = self.stream_feeder.take() else {
            return Err(Error::AlreadyStopped);
        };
        let Some(file_writer) = self.file_writer.take() else {
            return Err(Error::AlreadyStopped);
        };

        self.recorder.kill().await?;
        self.lame.wait().await?;

        let _ = stream_feeder.await;
        let _ = file_writer.await;

        Ok(())
    }
}

pub struct AudioRecorder {
    drive_client: GoogleDriveClient,
    dxe_client: DxeClient,

    config: HashMap<UnitId, AudioRecorderConfig>,

    tasks: Mutex<HashMap<BookingId, RecorderProcess>>,
}

impl AudioRecorder {
    pub async fn new(
        google_api_config: &GoogleApiConfig,
        config: HashMap<UnitId, AudioRecorderConfig>,
        dxe_client: DxeClient,
    ) -> Result<Self, Error> {
        let gcp_credential = CredentialManager::new(google_api_config, None)?;
        let drive_client = GoogleDriveClient::new(gcp_credential, &google_api_config.drive);

        Ok(Self {
            drive_client,
            dxe_client,

            config,

            tasks: Mutex::new(HashMap::new()),
        })
    }

    async fn start(
        &self,
        config: &AudioRecorderConfig,
        booking: &BookingWithUsers,
    ) -> Result<(), Error> {
        if self.tasks.lock().unwrap().contains_key(&booking.booking.id) {
            return Ok(());
        }

        let mut path = config.path_prefix.clone();
        path.push(format!("{}.mp3", booking.booking.id));

        let process = RecorderProcess::new(config, path).await?;

        self.tasks
            .lock()
            .unwrap()
            .insert(booking.booking.id, process);

        log::info!("Recording started for {}", booking.booking.id);

        Ok(())
    }

    async fn stop(&self, config: &AudioRecorderConfig, booking: &BookingWithUsers) {
        let Some(mut child) = self.tasks.lock().unwrap().remove(&booking.booking.id) else {
            return;
        };

        if let Err(e) = child.stop().await {
            log::warn!("Cannot stop recorder process: {e}");
        }

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
        let booking_id = booking.booking.id;
        tokio::task::spawn(async move {
            let url = match drive_client.upload(path.clone(), "audio/mp3").await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Couldn't upload file to Google Drive: {e}");
                    return;
                }
            };

            let expires_in = Utc::now() + TimeDelta::days(7);

            match dxe_client
                .post::<_, serde_json::Value>(
                    &format!("/booking/{booking_id}/recording"),
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

            let _ = tokio::fs::remove_file(path).await;
        });
    }

    async fn update(self: Arc<Self>) {
        let mut tasks = self.tasks.lock().unwrap();

        let mut tasks_to_collect = HashSet::new();
        for (key, task) in tasks.iter() {
            if task.has_stopped() {
                log::error!("Audio recording process for {key} exited prematurely.");
                tasks_to_collect.insert(*key);
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
    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            if let Some(config) = self.config.get(&event.booking.unit_id) {
                if let Err(e) = self.start(config, event).await {
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
    #[error("Could not upload to Google Drive: {0}")]
    Drive(#[from] DriveError),
    #[error("GCP credential error: {0}")]
    Gcp(#[from] GcpError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Recorder process has been already stopped.")]
    AlreadyStopped,
}
