mod kv_derive_with;

use chrono::{DateTime, Utc};
use fred::types::RedisKey;
use kv_derive::prelude::*;
use kv_derive::{FromMapping, IntoVec};
use serde::Deserialize;

#[derive(IntoVec, FromMapping, Deserialize, Debug)]
pub struct HardwareEntry {
    #[kv(
        rename = "ts",
        into_repr_with = "crate::kv_derive_with::to_timestamp",
        from_repr_with = "crate::kv_derive_with::from_timestamp"
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
