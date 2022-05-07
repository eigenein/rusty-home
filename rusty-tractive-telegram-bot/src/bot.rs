use std::borrow::Cow;
use std::time;

use anyhow::{Context, Result};
use async_std::task;
use fred::prelude::*;
use gethostname::gethostname;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods::Method;
use rusty_shared_telegram::{methods, models};
use tracing::{debug, error, info, instrument, warn};

pub struct Bot {
    redis: RedisClient,
    bot_api: BotApi,
    hostname: String,

    /// Redis key that stores the next offset for `getUpdates`.
    offset_key: String,

    get_updates_key: String,
}

impl Bot {
    const GET_UPDATES_TIMEOUT: time::Duration = time::Duration::from_secs(60);

    #[instrument(level = "info", skip_all, fields(bot_user_id = bot_user_id))]
    pub fn new(redis: RedisClient, bot_api: BotApi, bot_user_id: i64) -> Self {
        Self {
            redis,
            bot_api,
            hostname: gethostname().into_string().unwrap(),
            offset_key: format!("rusty:telegram:{}:offset", bot_user_id),
            get_updates_key: format!("rusty:telegram:{}:get_updates", bot_user_id),
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("setting up the bot…");
        methods::SetMyCommands::default()
            .command(models::BotCommand {
                command: Cow::Borrowed("start"),
                description: Cow::Borrowed("Tells your chat ID"),
            })
            .call(&self.bot_api)
            .await?;

        info!("running the bot…");
        loop {
            if self.claim_get_updates().await? {
                self.handle_updates().await?;
            } else {
                // Some other instance claimed the slot and is handling updates.
                // We'll sleep the next time slot would be available.
                let ttl_millis: u64 = self.redis.pttl(&self.get_updates_key).await?;
                debug!(ttl_millis = ttl_millis, "didn't manage to claim a slot");
                task::sleep(time::Duration::from_millis(ttl_millis + 1)).await;
            }
        }
    }

    /// The Bot API only allows one `getUpdates` long polling call at a time,
    /// but I want to run it on multiple hosts simultaneously to be more fault-tolerant.
    ///
    /// Thus, each instance will have to «claim» its `getUpdates` call before the actual
    /// call would be made.
    #[instrument(level = "info", skip_all)]
    async fn claim_get_updates(&self) -> Result<bool> {
        let response = self
            .redis
            .set::<Option<()>, _, _>(
                &self.get_updates_key,
                &self.hostname,
                Some(Expiration::EX(Self::GET_UPDATES_TIMEOUT.as_secs() as i64)),
                Some(SetOptions::NX),
                false,
            )
            .await;

        match response {
            Ok(Some(_)) => {
                debug!("claimed `getUpdates` slot");
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(error) => Err(error).context("failed to claim a slot"),
        }
    }

    async fn handle_updates(&self) -> Result<()> {
        let offset = self.get_offset().await?;
        let updates = methods::GetUpdates::new(Self::GET_UPDATES_TIMEOUT)
            .offset(offset)
            .allowed_update(methods::AllowedUpdate::Message)
            .call(&self.bot_api)
            .await?;

        for update in updates {
            info!(update.id = update.id);

            // I update the offset before calling `on_update` to avoid getting stuck
            // in case of a permanent error.
            self.set_offset(update.id + 1).await?;

            if let Err(error) = self.on_update(update.payload).await {
                error!(
                    update.id = update.id,
                    "failed to handle the update: {:#}", error
                );
            }
        }

        // Unclaim the time slot should we finish sooner.
        self.redis.del(&self.get_updates_key).await?;

        Ok(())
    }

    /// Retrieves the bot API offset from which we should read the updates.
    #[instrument(level = "debug", skip_all, fields(self.offset_key = self.offset_key.as_str()))]
    async fn get_offset(&self) -> Result<u64> {
        let offset = self
            .redis
            .get::<Option<u64>, _>(&self.offset_key)
            .await
            .context("failed to retrieve the offset")?
            .unwrap_or_default();
        Ok(offset)
    }

    #[instrument(level = "info", skip_all, fields(offset = offset))]
    async fn set_offset(&self, offset: u64) -> Result<()> {
        self.redis
            .set(&self.offset_key, offset, None, None, false)
            .await
            .context("failed to set the offset")
    }

    #[instrument(level = "info", skip_all)]
    async fn on_update(&self, payload: models::UpdatePayload) -> Result<()> {
        match payload {
            models::UpdatePayload::Message(message) => match message.text {
                Some(text) if text.starts_with("/start") => {
                    methods::SendMessage::new(
                        message.chat.id.into(),
                        format!(r#"👋 Your chat ID is `{}`\."#, message.chat.id),
                    )
                    .parse_mode(models::ParseMode::MarkdownV2)
                    .reply_to_message_id(message.id)
                    .call(&self.bot_api)
                    .await?;
                }
                _ => {
                    debug!(
                        message.text = ?message.text,
                        "ignoring the unsupported message"
                    );
                }
            },
            _ => {
                debug!(payload = ?payload, "ignoring the unsupported update");
            }
        }
        Ok(())
    }
}
