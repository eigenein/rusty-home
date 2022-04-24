use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};

#[must_use]
#[derive(Deserialize)]
pub struct Token {
    pub user_id: String,
    pub access_token: String,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    pub expires_at: DateTime<Utc>,
}

#[must_use]
#[derive(Debug, Deserialize)]
#[serde(tag = "message")]
pub enum Message {
    #[serde(rename = "handshake")]
    Handshake(HandshakeMessage),

    #[serde(rename = "keep-alive")]
    KeepAlive(KeepAliveMessage),

    #[serde(rename = "tracker_status")]
    TrackerStatus(TrackerStatusMessage),

    #[serde(other)]
    Other,
}

#[serde_as]
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct HandshakeMessage {
    pub channel_id: String,

    #[serde_as(as = "DurationSeconds<i64>")]
    pub keep_alive_ttl: Duration,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct KeepAliveMessage {
    #[serde(rename = "channelId")]
    pub channel_id: String,

    #[serde(
        rename = "keepAlive",
        deserialize_with = "chrono::serde::ts_seconds::deserialize"
    )]
    pub timestamp: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TrackerStatusMessage {
    pub tracker_id: String,

    pub hardware: Option<Hardware>,

    pub position: Option<Position>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Hardware {
    pub battery_level: u8,

    #[serde(
        rename = "time",
        deserialize_with = "chrono::serde::ts_seconds::deserialize"
    )]
    pub timestamp: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Position {
    pub accuracy: u32,
    pub course: u16,
    pub latlong: (f64, f64),

    #[serde(
        rename = "time",
        deserialize_with = "chrono::serde::ts_seconds::deserialize"
    )]
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::{bail, Result};
    use chrono::TimeZone;
    use serde_json::from_str;

    #[test]
    fn test_handshake_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"handshake","persistant":false,"channel_id":"channel_censored","keep_alive_ttl":600}"#,
        )?;
        match message {
            Message::Handshake(message) => {
                assert_eq!(message.channel_id, "channel_censored");
                assert_eq!(message.keep_alive_ttl, Duration::seconds(600));
                Ok(())
            }
            _ => bail!("incorrect message type: {:?}", message),
        }
    }

    #[test]
    fn test_keep_alive_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"keep-alive","channelId":"channel_censored","keepAlive":1650805106}"#,
        )?;
        match message {
            Message::KeepAlive(message) => {
                assert_eq!(message.channel_id, "channel_censored");
                assert_eq!(message.timestamp, Utc.timestamp(1650805106, 0));
                Ok(())
            }
            _ => bail!("incorrect message type: {:?}", message),
        }
    }

    #[test]
    fn test_tracker_status_initial_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"tracker_status","tracker_id":"CENSORED","tracker_state":"OPERATIONAL","position":{"time":1650802621,"latlong":[1.0,2.0],"sensor_used":"GPS","accuracy":2,"speed":0.2,"course":346,"time_rcvd":1650802623},"hardware":{"time":1650802598,"battery_level":55,"temperature_state":"NORMAL","power_saving_zone_id":null,"clip_mounted_state":false},"charging_state":"NOT_CHARGING","battery_state":"REGULAR","led_control":{"active":false,"timeout":300,"remaining":0,"pending":false,"reconnecting":false},"buzzer_control":{"active":false,"timeout":300,"remaining":0,"pending":false,"reconnecting":false},"live_tracking":{"active":false,"timeout":300,"remaining":0,"pending":false,"reconnecting":false},"pos_request":{"active":false,"timeout":300,"remaining":0,"pending":false,"reconnecting":false}}"#,
        )?;
        match message {
            Message::TrackerStatus(message) => {
                assert_eq!(message.tracker_id, "CENSORED");
                Ok(())
            }
            _ => bail!("incorrect message type: {:?}", message),
        }
    }

    #[test]
    fn test_tracker_status_regular_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"tracker_status","tracker_id":"CENSORED","tracker_state":"OPERATIONAL","position":{"time":1650806275,"latlong":[1.0,2.0],"sensor_used":"GPS","accuracy":22,"course":244,"altitude":-36,"time_rcvd":1650806276},"hardware":{"time":1650806276,"battery_level":51,"temperature_state":"NORMAL","power_saving_zone_id":null,"clip_mounted_state":false},"charging_state":"NOT_CHARGING","battery_state":"REGULAR"}"#,
        )?;
        match message {
            Message::TrackerStatus(message) => {
                assert_eq!(message.tracker_id, "CENSORED");
                Ok(())
            }
            _ => bail!("incorrect message type: {:?}", message),
        }
    }

    #[test]
    fn test_tracker_status_live_tracking_active_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"tracker_status","tracker_id":"CENSORED","tracker_state":"OPERATIONAL","charging_state":"NOT_CHARGING","battery_state":"REGULAR","live_tracking":{"active":true,"timeout":300,"remaining":299,"pending":false,"reconnecting":false,"started_at":1650802678}}"#,
        )?;
        match message {
            Message::TrackerStatus(message) => {
                assert_eq!(message.tracker_id, "CENSORED");
                Ok(())
            }
            _ => {
                bail!("incorrect message type: {:?}", message)
            }
        }
    }

    #[test]
    fn test_tracker_status_live_tracking_inactive_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"tracker_status","tracker_id":"CENSORED","tracker_state":"OPERATIONAL","charging_state":"NOT_CHARGING","battery_state":"REGULAR","live_tracking":{"active":false,"timeout":300,"remaining":0,"pending":false,"reconnecting":false}}"#,
        )?;
        match message {
            Message::TrackerStatus(message) => {
                assert_eq!(message.tracker_id, "CENSORED");
                Ok(())
            }
            _ => bail!("incorrect message type: {:?}", message),
        }
    }

    #[test]
    fn test_tracker_status_phone_sensor_ok() -> Result<()> {
        let message: Message = from_str(
            // language=json
            r#"{"message":"tracker_status","tracker_id":"CENSORED","tracker_state":"OPERATIONAL","position":{"time":1650837751,"latlong":[1.0,2.0],"sensor_used":"PHONE","accuracy":20,"speed":0.1,"course":314,"altitude":44,"nearby_user_id":"censored","time_rcvd":1650837757},"hardware":{"time":1650837553,"battery_level":96,"temperature_state":"NORMAL","power_saving_zone_id":null,"clip_mounted_state":false},"charging_state":"NOT_CHARGING","battery_state":"FULL"}"#,
        )?;
        match message {
            Message::TrackerStatus(message) => {
                assert_eq!(message.tracker_id, "CENSORED");
                Ok(())
            }
            _ => bail!("incorrect message type: {:?}", message),
        }
    }
}
