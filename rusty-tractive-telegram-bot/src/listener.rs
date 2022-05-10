use std::collections::HashMap;
use std::str::FromStr;
use std::time;

use anyhow::{anyhow, Context, Result};
use fred::prelude::*;
use fred::types::{RedisKey, XReadResponse, XID};
use gethostname::gethostname;
use rusty_shared_opts::heartbeat::Heartbeat;
use rusty_shared_redis::Redis;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods::Method;
use rusty_shared_telegram::{methods, models};
use tracing::{debug, info, instrument};

pub struct Listener {
    redis: Redis,
    bot_api: BotApi,
    heartbeat: Heartbeat,

    /// Target chat to which the updates will be posted.
    chat_id: models::ChatId,

    consumer_name: String,

    /// Redis stream consumer group name.
    group_name: String,

    /// Tractive position stream.
    position_stream_key: RedisKey,

    /// Tractive hardware position stream.
    hardware_stream_key: RedisKey,

    /// Stores the pinned message IDs, so that we can unpin them later.
    pinned_message_ids_key: String,

    /// Live location message ID, so that we can update it at any time.
    live_location_message_id_key: String,
}

impl Listener {
    const LIVE_PERIOD: time::Duration = time::Duration::from_secs(86400);

    pub async fn new(
        redis: Redis,
        bot_api: BotApi,
        heartbeat: Heartbeat,
        bot_user_id: i64,
        tracker_id: &str,
        chat_id: i64,
    ) -> Result<Self> {
        let group_name = format!("rusty:telegram:{}", bot_user_id);

        let position_stream_key = RedisKey::from(format!("rusty:tractive:{}:position", tracker_id));
        redis
            .create_consumer_group(&position_stream_key, &group_name)
            .await?;

        let hardware_stream_key = RedisKey::from(format!("rusty:tractive:{}:hardware", tracker_id));
        redis
            .create_consumer_group(&hardware_stream_key, &group_name)
            .await?;

        let this = Self {
            redis,
            bot_api,
            heartbeat,
            position_stream_key,
            hardware_stream_key,
            group_name,
            chat_id: models::ChatId::UniqueId(chat_id),
            consumer_name: gethostname().into_string().unwrap(),
            live_location_message_id_key: format!(
                "rusty:tractive:{}:telegram:{}:live_location_message_id",
                tracker_id, bot_user_id,
            ),
            pinned_message_ids_key: format!(
                "rusty:tractive:{}:telegram:{}:pinned_message_ids",
                tracker_id, bot_user_id,
            ),
        };
        Ok(this)
    }

    pub async fn run(self) -> Result<()> {
        info!("running the listener…");
        loop {
            self.handle_entries().await?;
            self.heartbeat.send().await;
        }
    }

    async fn handle_entries(&self) -> Result<()> {
        #[allow(clippy::mutable_key_type)]
        let response: XReadResponse<RedisKey, String, String, String> = self
            .redis
            .client
            .xreadgroup_map(
                &self.group_name,
                &self.consumer_name,
                None,
                Some(0),
                true,
                vec![&self.position_stream_key, &self.hardware_stream_key],
                XID::NewInGroup,
            )
            .await?;

        for (stream_id, entries) in response {
            info!(?stream_id, n_entries = entries.len());
            if stream_id == self.position_stream_key {
                for (entry_id, entry) in entries {
                    self.on_position_entry(&entry_id, entry).await?;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(entry_id = entry_id))]
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
            .client
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
                    .client
                    .set::<Option<()>, _, _>(
                        &self.live_location_message_id_key,
                        message_id,
                        Some(Expiration::EX(Self::LIVE_PERIOD.as_secs() as i64)),
                        Some(SetOptions::NX),
                        false,
                    )
                    .await?
                    .is_some()
                {
                    info!(message_id = message_id, "pinning the message…");
                    methods::PinChatMessage::new(&self.chat_id, message_id)
                        .disable_notification()
                        .call(&self.bot_api)
                        .await?;
                    self.delete_old_messages().await?;
                    self.redis
                        .client
                        .rpush(&self.pinned_message_ids_key, message_id)
                        .await?;
                } else {
                    info!(message_id = message_id, "too late – deleting the message…");
                    methods::DeleteMessage::new(&self.chat_id, message_id)
                        .call(&self.bot_api)
                        .await?;
                }
            }
        };

        Ok(())
    }

    #[instrument(skip_all)]
    async fn delete_old_messages(&self) -> Result<()> {
        while let Some(message_id) = self
            .redis
            .client
            .lpop::<Option<i64>, _>(&self.pinned_message_ids_key, None)
            .await?
        {
            info!(
                message_id = message_id,
                "unpinning and deleting the old message…",
            );
            methods::UnpinChatMessage::new(&self.chat_id, message_id)
                .call(&self.bot_api)
                .await?;
            methods::DeleteMessage::new(&self.chat_id, message_id)
                .call(&self.bot_api)
                .await?;
        }
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
