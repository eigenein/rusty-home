use anyhow::Result;
use clap::Parser;
use futures::future::try_join;
use rusty_shared_telegram::api::BotApi;
use sentry::integrations::anyhow::capture_anyhow;

use crate::bot::Bot;
use crate::listener::Listener;
use crate::opts::Opts;

mod bot;
mod listener;
mod opts;

#[async_std::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let _guard = opts.sentry.init();
    rusty_shared_tracing::init().unwrap();

    if let Err(error) = run(opts).await {
        capture_anyhow(&error);
    }
}

async fn run(opts: Opts) -> Result<()> {
    let bot_api = BotApi::new(opts.bot_token, std::time::Duration::from_secs(5))?;
    let me = bot_api.get_me().await?;

    let tracker_id = opts.tracker_id.to_lowercase();
    let bot = {
        let redis =
            rusty_shared_redis::connect(&opts.redis.addresses, opts.redis.service_name.clone())
                .await?;
        Bot::new(
            redis,
            bot_api.clone(),
            me.id,
            opts.heartbeat.get_heartbeat()?,
        )
    };
    let listener = {
        let redis =
            rusty_shared_redis::connect(&opts.redis.addresses, opts.redis.service_name).await?;
        Listener::new(redis, bot_api, me.id, &tracker_id, opts.chat_id).await?
    };
    try_join(bot.run(), listener.run()).await?;
    Ok(())
}
