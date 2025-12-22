use actix_http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder, web};
use async_compression::tokio::bufread::BrotliDecoder;
use chrono::TimeDelta;
use dxe_data::queries::booking::get_telemetry_file;
use dxe_s2s_shared::csv::{SoundMeterRow, Z2mPowerMeterRow};
use dxe_types::{BookingId, TelemetryType};
use plotters::backend::SVGBackend;
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::coord::combinators::IntoLogRange;
use plotters::drawing::IntoDrawingArea;
use plotters::series::LineSeries;
use plotters::style::{AsRelative, BLACK, RGBAColor};
use sqlx::SqlitePool;
use tokio::fs::File;
use tokio::io::BufReader;

use crate::config::TelemetryConfig;
use crate::models::Error;
use crate::models::handlers::admin::GetBookingTelemetryQuery;
use crate::utils::csv::read_csv;

pub async fn get(
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    query: web::Query<GetBookingTelemetryQuery>,
    config: web::Data<TelemetryConfig>,
) -> Result<HttpResponse, Error> {
    let mut connection = database.acquire().await?;

    let telemetry_file = get_telemetry_file(&mut connection, &booking_id, query.r#type)
        .await?
        .ok_or(Error::FileNotFound)?;

    let mut path = config.path.clone();
    path.push(telemetry_file.file_name);
    let file = File::open(&path)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;
    let reader = BufReader::new(file);
    let brotli_decoder = BrotliDecoder::new(reader);

    let mut svg_string = String::new();
    let svg = SVGBackend::with_string(&mut svg_string, (640, 360)).into_drawing_area();

    let mut chart = ChartBuilder::on(&svg);
    chart
        .set_label_area_size(LabelAreaPosition::Bottom, (8).percent())
        .set_label_area_size(LabelAreaPosition::Left, (16).percent())
        .margin((1).percent());

    match query.r#type {
        TelemetryType::PowerUsageRoom | TelemetryType::PowerUsageTotal => {
            match query.r#type {
                TelemetryType::PowerUsageRoom => {
                    chart.caption("Power usage (amplifiers)", ("sans-serif", (5).percent()));
                }
                TelemetryType::PowerUsageTotal => {
                    chart.caption("Power usage (cumulative)", ("sans-serif", (5).percent()));
                }
                _ => {}
            };

            let rows = read_csv::<Z2mPowerMeterRow, _>(brotli_decoder).await?;
            let last = rows
                .last()
                .map(|v| v.0)
                .unwrap_or(TimeDelta::milliseconds(0));

            let mut cum_min = -1.0;
            let mut cum_max = 0.0;
            let mut inst_min = -1.0;
            let mut inst_max = 0.0;

            for (_, row) in rows.iter() {
                if cum_min < 0.0 || row.power_usage_kwh < cum_min {
                    cum_min = row.power_usage_kwh;
                }
                if row.power_usage_kwh > cum_max {
                    cum_max = row.power_usage_kwh;
                }
                if inst_min < 0.0 || row.instantaneous_wattage < inst_min {
                    inst_min = row.instantaneous_wattage;
                }
                if row.instantaneous_wattage > inst_max {
                    inst_max = row.instantaneous_wattage;
                }
            }

            let mut chart = chart
                .right_y_label_area_size((16).percent())
                .build_cartesian_2d(TimeDelta::zero()..last, cum_min..cum_max)
                .map_err(|e| Error::Internal(Box::new(e)))?
                .set_secondary_coord(TimeDelta::milliseconds(0)..last, inst_min..inst_max);

            chart
                .configure_mesh()
                .y_desc("Power usage (kWh)")
                .x_desc("Time elapsed")
                .x_label_formatter(&|x| {
                    format!("{:02}:{:02}", x.num_minutes(), x.num_seconds() % 60)
                })
                .draw()
                .map_err(|e| Error::Internal(Box::new(e)))?;

            chart
                .configure_secondary_axes()
                .y_desc("Instantenous power usage (W)")
                .draw()
                .map_err(|e| Error::Internal(Box::new(e)))?;

            chart
                .draw_series(LineSeries::new(
                    rows.iter().map(|(time, row)| (*time, row.power_usage_kwh)),
                    BLACK,
                ))
                .map_err(|e| Error::Internal(Box::new(e)))?;
            chart
                .draw_secondary_series(LineSeries::new(
                    rows.iter()
                        .map(|(time, row)| (*time, row.instantaneous_wattage)),
                    RGBAColor(0, 0, 0, 0.3),
                ))
                .map_err(|e| Error::Internal(Box::new(e)))?;
        }
        TelemetryType::SoundMeter => {
            chart.caption(
                "Sound level (rehearsal room)",
                ("sans-serif", (5).percent()),
            );

            let rows = read_csv::<SoundMeterRow, _>(brotli_decoder).await?;
            let last = rows
                .last()
                .map(|v| v.0)
                .unwrap_or(TimeDelta::milliseconds(0));

            let min = 30.0;
            let mut max = 30.0;
            for (_, row) in rows.iter() {
                let level = row.decibel_level_10 as f64 / 10.0;
                if level > max {
                    max = level;
                }
            }

            let mut chart = chart
                .build_cartesian_2d(TimeDelta::zero()..last, (min..max).log_scale().base(2.0))
                .map_err(|e| Error::Internal(Box::new(e)))?;

            chart
                .configure_mesh()
                .y_desc("Sound level (dB)")
                .y_label_formatter(&|y| format!("{:.1}", y))
                .x_desc("Time elapsed")
                .x_label_formatter(&|x| {
                    format!("{:02}:{:02}", x.num_minutes(), x.num_seconds() % 60)
                })
                .draw()
                .map_err(|e| Error::Internal(Box::new(e)))?;

            chart
                .draw_series(LineSeries::new(
                    rows.iter()
                        .map(|(time, row)| (*time, row.decibel_level_10 as f64 / 10.0)),
                    BLACK,
                ))
                .map_err(|e| Error::Internal(Box::new(e)))?;
        }
    };

    svg.present().map_err(|e| Error::Internal(Box::new(e)))?;
    drop(svg);

    Ok(HttpResponseBuilder::new(StatusCode::OK)
        .content_type("image/svg+xml")
        .body(svg_string))
}
