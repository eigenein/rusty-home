use chrono::{DateTime, TimeZone, Utc};
use kv_derive::result::Result;

#[inline]
pub(crate) fn to_timestamp(datetime: DateTime<Utc>) -> i64 {
    datetime.timestamp()
}

#[inline]
pub(crate) fn from_timestamp(secs: i64) -> Result<DateTime<Utc>> {
    Ok(Utc.timestamp(secs, 0))
}
