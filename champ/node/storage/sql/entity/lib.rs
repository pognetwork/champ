use chrono::prelude::*;

mod entities;
use ::sea_orm::prelude::DateTimeUtc;
pub use entities::*;

pub mod sea_orm {
    pub use sea_orm::*;
}

pub fn unix_to_datetime(timestamp: u64) -> DateTimeUtc {
    let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime
}
