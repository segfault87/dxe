use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;
use std::time::Instant;

use csv_async::AsyncSerializer;
use serde::Serialize;
use tokio::fs::File;
use tokio::sync::Mutex;

struct FileHandle {
    path: PathBuf,
    started_at: Instant,
    csv: AsyncSerializer<File>,
}

#[derive(Serialize)]
pub struct Timestamp {
    pub elapsed: u128,
}

impl FileHandle {
    pub fn new(file: File, path: PathBuf) -> Self {
        Self {
            path,
            started_at: Instant::now(),
            csv: AsyncSerializer::from_writer(file),
        }
    }
}

pub struct TelemetryService<K> {
    path: PathBuf,
    documents: Mutex<HashMap<K, Mutex<FileHandle>>>,
}

impl<K: Hash + Eq + PartialEq + Display> TelemetryService<K> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            documents: Mutex::new(HashMap::new()),
        }
    }

    pub async fn start(&self, key: K, file_name: PathBuf) -> Result<(), Error> {
        let mut path = self.path.clone();
        path.push(file_name);
        let file = File::create(path.clone()).await?;

        self.documents
            .lock()
            .await
            .insert(key, Mutex::new(FileHandle::new(file, path)));

        Ok(())
    }

    pub async fn write<S: Serialize>(&self, key: &K, data: S) -> Result<(), Error> {
        let guard = self.documents.lock().await;
        let entry = guard.get(key).ok_or(Error::NotFound(key.to_string()))?;

        let mut entry_guard = entry.lock().await;

        let elapsed = entry_guard.started_at.elapsed().as_millis();

        entry_guard
            .csv
            .serialize((Timestamp { elapsed }, data))
            .await?;

        Ok(())
    }

    pub async fn stop(&self, key: &K) -> Result<PathBuf, Error> {
        let document = self
            .documents
            .lock()
            .await
            .remove(key)
            .ok_or(Error::NotFound(key.to_string()))?;

        let mut guard = document.lock().await;
        guard.csv.flush().await?;

        Ok(guard.path.clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV document for key {0} not found.")]
    NotFound(String),
    #[error("Error saving CSV row: {0}")]
    Csv(#[from] csv_async::Error),
    #[error("Error serializing data: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
