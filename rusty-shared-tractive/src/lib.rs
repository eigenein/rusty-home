use chrono::{DateTime, TimeZone, Utc};
use fred::types::RedisKey;
use kv_derive::prelude::*;
use kv_derive::{FromMapping, IntoVec};
use serde::Deserialize;

#[derive(IntoVec, FromMapping, Deserialize, Debug)]
pub struct HardwareEntry {
    #[kv(
        rename = "ts",
        into_repr_with = "|timestamp: DateTime<Utc>| timestamp.timestamp()",
        from_repr_with = "|secs: i64| kv_derive::result::Result::Ok(Utc.timestamp(secs, 0))"
    )]
    #[serde(
        rename = "time",
        deserialize_with = "chrono::serde::ts_seconds::deserialize"
    )]
    pub timestamp: DateTime<Utc>,

    #[kv(rename = "battery")]
    pub battery_level: u8,
}

pub fn hardware_stream_key(tracker_id: &str) -> RedisKey {
    RedisKey::from(format!("rusty:tractive:{}:hardware", tracker_id.to_lowercase()))
}

pub fn position_stream_key(tracker_id: &str) -> RedisKey {
    RedisKey::from(format!("rusty:tractive:{}:position", tracker_id.to_lowercase()))
}
