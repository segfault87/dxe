use actix_web::web;

#[derive(serde::Serialize)]
pub struct Timestamp {
    timestamp: i64,
}

pub async fn get() -> web::Json<Timestamp> {
    let now = chrono::Utc::now();
    web::Json(Timestamp {
        timestamp: now.timestamp_millis(),
    })
}
