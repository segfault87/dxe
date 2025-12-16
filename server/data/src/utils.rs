use chrono::{DateTime, Utc};

pub fn is_in_effect(datetime: &Option<DateTime<Utc>>, now: &DateTime<Utc>) -> bool {
    datetime.map(|v| &v < now).unwrap_or(false)
}
