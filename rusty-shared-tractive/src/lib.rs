use chrono::{DateTime, TimeZone, Utc};
use fred::types::RedisKey;
use kv_derive::prelude::*;
use kv_derive::{FromMapping, IntoVec};

#[derive(IntoVec, FromMapping)]
pub struct HardwareEntry {
    #[kv(
        rename = "ts",
        into_repr_with = "|timestamp: DateTime<Utc>| timestamp.timestamp()",
        from_repr_with = "|secs: i64| kv_derive::result::Result::Ok(Utc.timestamp(secs, 0))"
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
