#![warn(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::trivially_copy_pass_by_ref,
    clippy::map_unwrap_or,
    clippy::explicit_into_iter_loop,
    clippy::unused_self,
    clippy::needless_pass_by_value
)]

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

#[derive(IntoVec, FromMapping, Debug)]
pub struct PositionEntry {
    #[kv(
        rename = "ts",
        into_repr_with = "crate::kv_derive_with::to_timestamp",
        from_repr_with = "crate::kv_derive_with::from_timestamp"
    )]
    pub timestamp: DateTime<Utc>,

    #[kv(rename = "lat")]
    pub latitude: f64,

    #[kv(rename = "lon")]
    pub longitude: f64,

    pub accuracy: u32,

    #[kv(optional, default())]
    pub course: Option<u16>,
}

pub fn hardware_stream_key(tracker_id: &str) -> RedisKey {
    RedisKey::from(format!("rusty:tractive:{}:hardware", tracker_id.to_lowercase()))
}

pub fn position_stream_key(tracker_id: &str) -> RedisKey {
    RedisKey::from(format!("rusty:tractive:{}:position", tracker_id.to_lowercase()))
}
