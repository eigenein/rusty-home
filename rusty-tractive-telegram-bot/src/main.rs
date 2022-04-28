use anyhow::{Context, Result};
use clap::Parser;
use fred::prelude::*;
use tracing::{debug, error, info, instrument};

use rusty_shared_opts::heartbeat::Heartbeat;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::models;

use crate::opts::Opts;

mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = opts.sentry.init();
    rusty_shared_tracing::init()?;
    Main::new(opts).await?.serve().await;
    Ok(())
}

struct Main {
    heartbeat: Heartbeat,
    redis: RedisClient,
    bot_api: BotApi,

    stream_key: String,

    /// Redis key that stores the next offset for `getUpdates`.
    offset_key: String,

    /// Redis stream group name.
    group_name: String,
}

impl Main {
    const GET_UPDATES_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

    #[instrument(level = "info", skip_all, fields(opts.tracker_id = opts.tracker_id.as_str()))]
    async fn new(opts: Opts) -> Result<Self> {
        let bot_api = BotApi::new(opts.bot_token, std::time::Duration::from_secs(5))?;
        let me = bot_api.get_me().await?;
        let tracker_id = opts.tracker_id.to_lowercase();
        info!(me.id = me.id, tracker_id = tracker_id.as_str());

        let redis =
            rusty_shared_redis::connect(opts.redis.addresses, opts.redis.service_name).await?;
        let stream_key = format!("rusty:tractive:{}:position", tracker_id);
        let group_name = format!("rusty:telegram:{}", me.id);
        rusty_shared_redis::create_consumer_group(&redis, &stream_key, &group_name).await?;

        Ok(Self {
            heartbeat: opts.heartbeat.get_heartbeat()?,
            redis,
            bot_api,
            offset_key: format!("rusty:tractive:{}:telegram:{}:offset", tracker_id, me.id),
            group_name,
            stream_key,
        })
    }

    async fn serve(self) -> ! {
        info!("running…");
        loop {
            if let Err(error) = self.loop_().await {
                error!("main loop error: {:#}", error);
            } else {
                self.heartbeat.send().await;
            }
        }
    }

    async fn loop_(&self) -> Result<()> {
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

    #[instrument(level = "debug", skip_all, fields(offset = offset))]
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
