use chrono::{DateTime, NaiveTime, TimeZone, Timelike, Utc};

pub fn truncate_time<Tz: TimeZone>(datetime: DateTime<Tz>) -> DateTime<Tz> {
    datetime
        .with_time(NaiveTime::from_hms_opt(datetime.hour(), 0, 0).unwrap())
        .unwrap()
}

pub fn is_in_effect(datetime: &Option<DateTime<Utc>>, now: &DateTime<Utc>) -> bool {
    datetime.map(|v| &v < now).unwrap_or(false)
}
