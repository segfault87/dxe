use actix_multipart::form::MultipartForm;
use actix_multipart::form::json::Json as MultipartJson;
use actix_multipart::form::tempfile::TempFile;
use actix_web::web;
use dxe_data::queries::booking::create_telemetry_file;
use dxe_s2s_shared::handlers::UploadTelemetryFileRequest;
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::{config::TelemetryConfig, middleware::datetime_injector::Now, models::Error};

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "5MB")]
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
    let Some(file_name) = form.file.file_name.clone() else {
        return Err(Error::BadFileUpload);
    };

    let mut path = telemetry_config.path.clone();
    path.push(file_name.clone());

    form.file
        .file
        .persist(path)
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
