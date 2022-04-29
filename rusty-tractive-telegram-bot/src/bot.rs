use anyhow::{Context, Result};
use fred::prelude::*;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::models;
use tracing::{debug, error, info, instrument};

pub struct Bot {
    redis: RedisClient,
    bot_api: BotApi,

    /// Redis key that stores the next offset for `getUpdates`.
    offset_key: String,
}

impl Bot {
    const GET_UPDATES_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

    #[instrument(level = "info", skip_all, fields(bot_user_id = bot_user_id, tracker_id = tracker_id))]
    pub fn new(redis: RedisClient, bot_api: BotApi, bot_user_id: i64, tracker_id: &str) -> Self {
        Self {
            redis,
            bot_api,
            offset_key: format!(
                "rusty:tractive:{}:telegram:{}:offset",
                tracker_id, bot_user_id
            ),
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("running the bot…");
        loop {
            if let Err(error) = self.handle_updates().await {
                error!("main loop error: {:#}", error);
            }
        }
    }

    async fn handle_updates(&self) -> Result<()> {
        let offset = self.get_offset().await?;
        let updates = self
            .bot_api
            .get_updates(offset, Self::GET_UPDATES_TIMEOUT)
            .await?;

        for update in updates {
            info!(update.id = update.id);
            self.on_update(update.payload).await?;
            self.set_offset(update.id + 1).await?;
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all, fields(self.offset_key = self.offset_key.as_str()))]
    async fn get_offset(&self) -> Result<i64> {
        let offset = self
            .redis
            .get::<Option<i64>, _>(&self.offset_key)
            .await
            .context("failed to retrieve the offset")?
            .unwrap_or_default();
        Ok(offset)
    }

    #[instrument(level = "info", skip_all, fields(offset = offset))]
    async fn set_offset(&self, offset: i64) -> Result<()> {
        self.redis
            .set(&self.offset_key, offset, None, None, false)
            .await
            .context("failed to set the offset")
    }

    #[instrument(level = "info", skip_all)]
    async fn on_update(&self, payload: models::UpdatePayload) -> Result<()> {
        match payload {
            _ => {
                debug!("ignoring the unsupported update");
                Ok(())
            }
        }
    }
}
