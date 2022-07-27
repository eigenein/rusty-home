//! Implements Redis stream listener.

use std::collections::HashMap;
use std::time;

use fred::prelude::*;
use fred::types::{RedisKey, XReadResponse, XID};
use gethostname::gethostname;
use rusty_shared_opts::heartbeat::Heartbeat;
use rusty_shared_redis::Redis;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods::*;
use rusty_shared_telegram::models::*;
use rusty_shared_tractive::*;

use crate::opts::BatteryOpts;
use crate::prelude::*;

pub struct Listener {
    redis: Redis,
    bot_api: BotApi,
    heartbeat: Heartbeat,
    battery_opts: BatteryOpts,

    /// Target chat to which the updates will be posted.
    chat_id: ChatId,

    /// Consumer name within the Redis group.
    consumer_name: String,

    /// Redis stream consumer group name.
    group_name: String,

    keys: RedisKeys,
}

struct RedisKeys {
    /// Tractive position stream.
    position_stream: RedisKey,

    /// Tractive hardware position stream.
    hardware_stream: RedisKey,

    /// Stores the pinned message IDs, so that we can unpin them later.
    pinned_message_ids: RedisKey,

    /// Live location message ID, so that we can update it at any time.
    live_location_message_id: RedisKey,

    last_known_battery_level: RedisKey,
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
        battery_opts: BatteryOpts,
    ) -> Result<Self> {
        let group_name = format!("rusty:telegram:{}", bot_user_id);

        let position_stream_key = position_stream_key(tracker_id);
        redis
            .create_consumer_group(&position_stream_key, &group_name)
            .await?;

        let hardware_stream_key = hardware_stream_key(tracker_id);
        redis
            .create_consumer_group(&hardware_stream_key, &group_name)
            .await?;

        let this = Self {
            redis,
            bot_api,
            heartbeat,
            group_name,
            chat_id: ChatId::UniqueId(chat_id),
            consumer_name: gethostname().into_string().unwrap(),
            battery_opts,
            keys: RedisKeys {
                position_stream: position_stream_key,
                hardware_stream: hardware_stream_key,
                live_location_message_id: RedisKey::from(format!(
                    "rusty:tractive:{}:telegram:{}:live_location_message_id",
                    tracker_id, bot_user_id,
                )),
                pinned_message_ids: RedisKey::from(format!(
                    "rusty:tractive:{}:telegram:{}:pinned_message_ids",
                    tracker_id, bot_user_id,
                )),
                last_known_battery_level: RedisKey::from(format!(
                    "rusty:tractive:{}:telegram:{}:last_known_battery_level",
                    tracker_id, bot_user_id,
                )),
            },
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
                Some(0), // FIXME: set `block` and timeout on the call to prevent the freezes.
                true,
                vec![&self.keys.position_stream, &self.keys.hardware_stream],
                vec![XID::NewInGroup, XID::NewInGroup],
            )
            .await?;

        for (stream_id, entries) in response {
            info!(stream_id = ?stream_id.inner(), n_entries = entries.len());
            if stream_id == self.keys.position_stream {
                for (entry_id, entry) in entries {
                    self.on_position_entry(&entry_id, entry.try_into()?).await?;
                }
            } else if stream_id == self.keys.hardware_stream {
                for (entry_id, entry) in entries {
                    self.on_hardware_entry(&entry_id, entry.try_into()?).await?;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(entry_id = _entry_id))]
    async fn on_position_entry(&self, _entry_id: &str, entry: PositionEntry) -> Result<()> {
        debug!(entry = ?entry);
        let location = Location::new(self.chat_id.clone(), entry.latitude, entry.longitude)
            .horizontal_accuracy(entry.accuracy as f32)
            .heading(entry.course);
        info!(
            latitude = location.latitude,
            longitude = location.longitude,
            horizontal_accuracy = location.horizontal_accuracy,
            heading = location.heading,
            "new location entry",
        );

        match self
            .redis
            .client
            .get::<Option<i64>, _>(&self.keys.live_location_message_id)
            .await?
        {
            Some(message_id) => {
                debug!(message_id, "editing existing message…");
                if let Err(error) =
                    EditMessageLiveLocation::new(self.chat_id.clone(), message_id, location)
                        .call(&self.bot_api)
                        .await
                {
                    error!("failed to edit the live location: {:#}", error);
                }
            }
            None => {
                info!("sending a new message…");
                let message_id = SendLocation::new(location)
                    .live_period(Self::LIVE_PERIOD)
                    .call(&self.bot_api)
                    .await?
                    .id;
                debug!(message_id, "updating the live location message ID…");
                if self
                    .redis
                    .client
                    .set::<Option<()>, _, _>(
                        &self.keys.live_location_message_id,
                        message_id,
                        Some(Expiration::EX(Self::LIVE_PERIOD.as_secs() as i64)),
                        Some(SetOptions::NX),
                        false,
                    )
                    .await?
                    .is_some()
                {
                    info!(message_id, "pinning the message…");
                    PinChatMessage::new(&self.chat_id, message_id)
                        .disable_notification()
                        .call(&self.bot_api)
                        .await?;
                    self.delete_old_messages().await?;
                    self.redis
                        .client
                        .rpush(&self.keys.pinned_message_ids, message_id)
                        .await?;
                } else {
                    info!(message_id, "too late – deleting the message…");
                    DeleteMessage::new(&self.chat_id, message_id)
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
            .lpop::<Option<i64>, _>(&self.keys.pinned_message_ids, None)
            .await?
        {
            info!(message_id, "unpinning and deleting the old message…");
            UnpinChatMessage::new(&self.chat_id, message_id)
                .call(&self.bot_api)
                .await?;
            if let Err(error) = DeleteMessage::new(&self.chat_id, message_id)
                .call(&self.bot_api)
                .await
            {
                error!("failed to delete the old message: {:#}", error);
            }
        }
        Ok(())
    }

    #[instrument(skip_all, fields(entry_id = _entry_id))]
    async fn on_hardware_entry(&self, _entry_id: &str, entry: HardwareEntry) -> Result<()> {
        let battery_level: u8 = entry.battery_level;
        info!(battery_level, "new hardware entry");
        let (is_updated, last_level) = self
            .redis
            .set_if_not_equal(&self.keys.last_known_battery_level, battery_level)
            .await?;
        if is_updated {
            self.on_battery_level_changed(last_level, battery_level)
                .await?;
        }
        Ok(())
    }

    #[instrument(skip_all, fields(current_level = current_level, last_level = last_level))]
    async fn on_battery_level_changed(
        &self,
        last_level: Option<u8>,
        current_level: u8,
    ) -> Result<()> {
        info!(current_level, last_level, "battery level changed");
        let last_level = last_level.unwrap_or(current_level);
        let template_values = HashMap::from([("current_level", current_level.to_string())]);

        let text = if current_level >= self.battery_opts.full_level
            && last_level < self.battery_opts.full_level
        {
            self.battery_opts.full_message.0.render(&template_values)?
        } else if current_level <= self.battery_opts.low_level
            && last_level > self.battery_opts.low_level
        {
            self.battery_opts.low_message.0.render(&template_values)?
        } else if current_level <= self.battery_opts.critical_level {
            self.battery_opts
                .critical_message
                .0
                .render(&template_values)?
        } else {
            return Ok(());
        };
        SendMessage::new(&self.chat_id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .call(&self.bot_api)
            .await
            .context("failed to send the battery notification")?;
        Ok(())
    }
}
