use anyhow::Result;
use clap::Parser;
use futures::future::try_join;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods;
use rusty_shared_telegram::methods::Method;
use std::time::Duration;

use crate::bot::Bot;
use crate::listener::Listener;
use crate::opts::Opts;

mod bot;
mod listener;
mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = rusty_shared_tracing::init(opts.sentry).unwrap();

    let bot_api = BotApi::new(opts.service.bot_token, Duration::from_secs(5))?;
    let me = methods::GetMe.call(&bot_api).await?;
    let redis = rusty_shared_redis::Redis::connect(&opts.redis.redis_url).await?;

    let bot = Bot::new(redis.clone().await?, bot_api.clone(), me.id);
    let listener = Listener::new(
        redis,
        bot_api,
        opts.heartbeat.get_heartbeat()?,
        me.id,
        &opts.service.tracker_id.to_lowercase(),
        opts.service.chat_id,
        opts.service.battery,
    )
    .await?;

    try_join(bot.run(), listener.run()).await?;
    Ok(())
}
