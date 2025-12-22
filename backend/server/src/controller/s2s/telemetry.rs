use actix_multipart::form::MultipartForm;
use actix_multipart::form::json::Json as MultipartJson;
use actix_multipart::form::tempfile::TempFile;
use actix_web::web;
use async_compression::tokio::write::BrotliEncoder;
use dxe_data::queries::booking::create_telemetry_file;
use dxe_s2s_shared::handlers::UploadTelemetryFileRequest;
use dxe_types::BookingId;
use sqlx::SqlitePool;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::config::TelemetryConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::Error;

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "1MB")]
    file: TempFile,
    request: MultipartJson<UploadTelemetryFileRequest>,
}

pub async fn post(
    now: Now,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    telemetry_config: web::Data<TelemetryConfig>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let Some(mut file_name) = form.file.file_name.clone() else {
        return Err(Error::BadFileUpload);
    };
    file_name.push_str(".br");

    let mut path = telemetry_config.path.clone();
    path.push(file_name.clone());

    let mut temp_file = File::from_std(form.file.file.into_file());
    let mut file = File::create(path)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;

    let mut writer = BrotliEncoder::new(&mut file);
    let mut buffer = [0u8; 65536];
    while let bytes = temp_file
        .read(&mut buffer)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?
        && bytes > 0
    {
        writer
            .write(&buffer[0..bytes])
            .await
            .map_err(|e| Error::Internal(Box::new(e)))?;
    }

    writer
        .flush()
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;

    let mut tx = database.begin().await?;

    create_telemetry_file(
        &mut tx,
        &now,
        booking_id.as_ref(),
        form.request.r#type,
        file_name,
    )
    .await?;

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
