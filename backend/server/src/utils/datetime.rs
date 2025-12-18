use chrono::{DateTime, NaiveTime, TimeZone, Timelike};

pub use dxe_data::utils::is_in_effect;

pub fn truncate_time<Tz: TimeZone>(datetime: DateTime<Tz>) -> DateTime<Tz> {
    datetime
        .with_time(NaiveTime::from_hms_opt(datetime.hour(), 0, 0).unwrap())
        .unwrap()
}
