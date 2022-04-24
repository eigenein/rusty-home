use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "message")]
enum Message {
    #[serde(rename = "keep-alive")]
    KeepAlive(KeepAliveMessage),

    #[serde(other)]
    Other,
}

#[derive(Debug, PartialEq, Deserialize)]
struct KeepAliveMessage {
    #[serde(rename = "channelId")]
    channel_id: String,

    #[serde(
        rename = "keepAlive",
        deserialize_with = "chrono::serde::ts_seconds::deserialize"
    )]
    timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::TimeZone;

    #[test]
    fn test_keep_alive_ok() -> Result<()> {
        let message: Message = serde_json::from_str(
            // language=json
            r#"{"message":"keep-alive","channelId":"channel_censored","keepAlive":1650805106}"#,
        )?;
        assert_eq!(
            message,
            Message::KeepAlive(KeepAliveMessage {
                channel_id: "channel_censored".to_string(),
                timestamp: Utc.timestamp(1650805106, 0),
            }),
        );
        Ok(())
    }
}
