use std::collections::HashMap;
use std::str::FromStr;
use std::time;

use anyhow::{anyhow, Context, Result};
use fred::prelude::*;
use fred::types::{XReadResponse, XID};
use gethostname::gethostname;
use rusty_shared_opts::heartbeat::Heartbeat;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods::Method;
use rusty_shared_telegram::{methods, models};
use tracing::{debug, error, info, instrument};

pub struct Listener {
    redis: RedisClient,
    bot_api: BotApi,
    heartbeat: Heartbeat,

    /// Target chat to which the updates will be posted.
    chat_id: models::ChatId,

    consumer_name: String,

    /// Redis stream consumer group name.
    group_name: String,

    position_stream_key: String,

    live_location_message_id_key: String,
}

impl Listener {
    const LIVE_PERIOD: time::Duration = time::Duration::from_secs(86400);

    pub async fn new(
        redis: RedisClient,
        bot_api: BotApi,
        heartbeat: Heartbeat,
        bot_user_id: i64,
        tracker_id: &str,
        chat_id: i64,
    ) -> Result<Self> {
        let position_stream_key = format!("rusty:tractive:{}:position", tracker_id);
        let group_name = format!("rusty:telegram:{}", bot_user_id);
        rusty_shared_redis::create_consumer_group(&redis, &position_stream_key, &group_name)
            .await?;

        let this = Self {
            redis,
            bot_api,
            heartbeat,
            position_stream_key,
            group_name,
            chat_id: models::ChatId::UniqueId(chat_id),
            consumer_name: gethostname().into_string().unwrap(),
            live_location_message_id_key: format!(
                "rusty:tractive:{}:telegram:{}:live_location_message_id",
                tracker_id, bot_user_id,
            ),
        };
        Ok(this)
    }

    pub async fn run(self) -> Result<()> {
        info!("running the listener…");
        loop {
            if let Err(error) = self.handle_entries().await {
                error!("stream listener error: {:#}", error);
            } else {
                self.heartbeat.send().await;
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
                true,
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
        fields: HashMap<String, String>,
    ) -> Result<()> {
        debug!(fields = ?fields);
        let location = methods::Location::new(
            self.chat_id.clone(),
            get_parsed(&fields, "lat")?,
            get_parsed(&fields, "lon")?,
        );
        let location = location.horizontal_accuracy(get_parsed(&fields, "accuracy")?);
        let location = match fields.get("course") {
            Some(course) => location.heading(course.parse::<u16>()?),
            None => location,
        };
        info!(
            latitude = location.latitude,
            longitude = location.longitude,
            horizontal_accuracy = location.horizontal_accuracy,
            heading = location.heading,
        );

        match self
            .redis
            .get::<Option<i64>, _>(&self.live_location_message_id_key)
            .await?
        {
            Some(message_id) => {
                debug!(message_id = message_id, "editing existing message…");
                methods::EditMessageLiveLocation::new(self.chat_id.clone(), message_id, location)
                    .call(&self.bot_api)
                    .await?;
            }
            None => {
                info!("sending a new message…");
                let message_id = methods::SendLocation::new(location)
                    .live_period(Self::LIVE_PERIOD)
                    .call(&self.bot_api)
                    .await?
                    .id;
                debug!(
                    message_id = message_id,
                    "updating the live location message ID…",
                );
                if self
                    .redis
                    .set::<Option<()>, _, _>(
                        &self.live_location_message_id_key,
                        message_id,
                        Some(Expiration::EX(Self::LIVE_PERIOD.as_secs() as i64)),
                        Some(SetOptions::NX),
                        false,
                    )
                    .await?
                    .is_none()
                {
                    info!(message_id = message_id, "deleting our message…");
                    methods::DeleteMessage {
                        chat_id: self.chat_id.clone(),
                        message_id,
                    }
                    .call(&self.bot_api)
                    .await?;
                } else {
                    info!(message_id = message_id, "pinning the message…");
                    methods::PinChatMessage {
                        chat_id: self.chat_id.clone(),
                        message_id,
                        disable_notification: true,
                    }
                    .call(&self.bot_api)
                    .await?;
                }
            }
        };

        Ok(())
    }
}

// TODO: move to a shared package.
fn get_parsed<T>(fields: &HashMap<String, String>, key: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    fields
        .get(key)
        .ok_or_else(|| anyhow!("missing `{}`", key))?
        .parse()
        .with_context(|| format!("failed to parse `{}`", key))
}
