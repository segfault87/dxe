use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use futures::future::OptionFuture;
use futures::{Stream, StreamExt};
use tasi_sound_level_meter::TasiSoundLevelMeter;

use crate::config::SoundMeterConfig;
use crate::tables::{QualifiedPath, TablePublisher};
use crate::types::{DeviceId, DeviceRef, DeviceType, PublishKey};

#[pin_project::pin_project]
pub struct Tasi653bSoundLevelMeter {
    serial_number: Option<String>,
    #[pin]
    meter: TasiSoundLevelMeter,
    sleep: Pin<Box<OptionFuture<tokio::time::Sleep>>>,
}

impl Tasi653bSoundLevelMeter {
    pub fn new(serial_number: Option<String>) -> Result<Self, tasi_sound_level_meter::Error> {
        let meter = TasiSoundLevelMeter::new(serial_number.clone())?;

        Ok(Self {
            serial_number,
            meter,
            sleep: Box::pin(None.into()),
        })
    }
}

impl Stream for Tasi653bSoundLevelMeter {
    type Item = f64;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();

        match this.sleep.as_mut().poll(cx) {
            Poll::Ready(Some(())) => {
                this.sleep.set(None.into());
            }
            Poll::Ready(None) => {}
            Poll::Pending => {
                return Poll::Pending;
            }
        }

        match this.meter.as_mut().poll_next(cx) {
            Poll::Ready(Some(value)) => Poll::Ready(Some(value as f64 / 10.0)),
            Poll::Ready(None) => {
                log::warn!("Sound meter disconnected. reconnecting...");
                match TasiSoundLevelMeter::new(this.serial_number.clone()) {
                    Ok(v) => {
                        this.meter.set(v);
                    }
                    Err(e) => {
                        log::error!("Could not initialize sound meter: {e}");
                        this.sleep
                            .set(Some(tokio::time::sleep(Duration::from_secs(1))).into());
                    }
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[pin_project::pin_project(project = SoundMeterDeviceBackendProjection)]
pub enum SoundMeterDeviceBackend {
    Tasi653b(#[pin] Tasi653bSoundLevelMeter),
}

impl Stream for SoundMeterDeviceBackend {
    type Item = f64;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.project() {
            SoundMeterDeviceBackendProjection::Tasi653b(meter) => meter.poll_next(cx),
        }
    }
}

pub struct SoundMeterDevice {
    publish_key: PublishKey,
}

impl SoundMeterDevice {
    pub fn new(config: &SoundMeterConfig) -> Result<(Self, SoundMeterDeviceBackend), Error> {
        let backend = match &config.device {
            crate::config::SoundMeterDevice::Tasi653b { serial_number } => {
                SoundMeterDeviceBackend::Tasi653b(Tasi653bSoundLevelMeter::new(
                    serial_number.clone(),
                )?)
            }
        };

        Ok((
            Self {
                publish_key: config.publish_key.clone(),
            },
            backend,
        ))
    }
}

pub struct SoundMeterPath;

impl QualifiedPath for SoundMeterPath {
    type TableKey = DeviceId;
    type Path = DeviceRef;

    fn path(table_key: &Self::TableKey) -> Self::Path {
        DeviceRef {
            r#type: DeviceType::SoundMeter,
            id: table_key.clone(),
        }
    }
}

pub struct SoundMeterController {
    table: TablePublisher<DeviceId, DeviceRef, SoundMeterPath>,
}

impl SoundMeterController {
    pub fn new<'a>(
        config: impl Iterator<Item = &'a SoundMeterConfig>,
    ) -> Result<(Self, Vec<tokio::task::JoinHandle<()>>), Error> {
        let mut tasks = vec![];

        let table = TablePublisher::new();

        for config in config {
            let (device, mut backend) = SoundMeterDevice::new(config)?;
            let publish_key = device.publish_key.clone();
            let device_id = config.id.clone();

            let table_inner = table.clone();

            let task = tokio::task::spawn(async move {
                while let Some(value) = backend.next().await {
                    table_inner.update_value(
                        device_id.clone(),
                        publish_key.clone(),
                        serde_json::Number::from_f64(value).into(),
                    );
                }
            });

            tasks.push(task);
        }

        Ok((Self { table }, tasks))
    }

    pub fn publisher(&self) -> TablePublisher<DeviceId, DeviceRef, SoundMeterPath> {
        self.table.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error initializing sound meter: {0}")]
    Tasi(#[from] tasi_sound_level_meter::Error),
}
