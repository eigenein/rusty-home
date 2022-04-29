use std::collections::HashMap;

use anyhow::{anyhow, Result};
use fred::prelude::*;
use fred::types::{XReadResponse, XID};
use gethostname::gethostname;
use rusty_shared_telegram::api::BotApi;
use tracing::{error, info, instrument};

pub struct Listener {
    redis: RedisClient,
    bot_api: BotApi,
    position_stream_key: String,

    consumer_name: String,

    /// Redis stream consumer group name.
    group_name: String,
}

impl Listener {
    pub async fn new(
        redis: RedisClient,
        bot_api: BotApi,
        bot_user_id: i64,
        tracker_id: &str,
    ) -> Result<Self> {
        let position_stream_key = format!("rusty:tractive:{}:position", tracker_id);
        let group_name = format!("rusty:telegram:{}", bot_user_id);
        rusty_shared_redis::create_consumer_group(&redis, &position_stream_key, &group_name)
            .await?;

        let this = Self {
            redis,
            bot_api,
            position_stream_key,
            group_name,
            consumer_name: gethostname().into_string().unwrap(),
        };
        Ok(this)
    }

    pub async fn run(self) -> Result<()> {
        info!("running the listener…");
        loop {
            if let Err(error) = self.handle_entries().await {
                error!("stream listener error: {:#}", error);
            }
        }
    }

    async fn handle_entries(&self) -> Result<()> {
        let response: XReadResponse<String, String, String, String> = self
            .redis
            .xreadgroup_map(
                &self.group_name,
                &self.consumer_name,
                None,
                Some(0),
                false,
                &self.position_stream_key,
                XID::NewInGroup,
            )
            .await?;

        for (stream_id, entries) in response {
            if stream_id == self.position_stream_key {
                for (entry_id, entry) in entries {
                    if let Err(error) = self.on_position_entry(&entry_id, entry).await {
                        error!(
                            entry_id = entry_id.as_str(),
                            "failed to handle the stream entry: {:#}", error,
                        );
                    } else {
                        self.redis
                            .xack(&stream_id, &self.group_name, entry_id)
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }

    #[instrument(level = "info", skip_all, fields(entry_id = entry_id))]
    async fn on_position_entry(
        &self,
        entry_id: &str,
        entry: HashMap<String, String>,
    ) -> Result<()> {
        let _latitude: f64 = entry
            .get("lat")
            .ok_or_else(|| anyhow!("missing latitude"))?
            .parse()?;
        let _longitude: f64 = entry
            .get("lon")
            .ok_or_else(|| anyhow!("missing longitude"))?
            .parse()?;
        let _accuracy: u32 = entry
            .get("accuracy")
            .ok_or_else(|| anyhow!("missing accuracy"))?
            .parse()?;
        let _course = match entry.get("course") {
            Some(course) => Some(course.parse::<u16>()?),
            None => None,
        };
        Ok(())
    }
}
