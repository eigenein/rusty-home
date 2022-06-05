use anyhow::Result;
use clap::Parser;
use futures::future::try_join;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods;
use rusty_shared_telegram::methods::Method;
use tracing::error;

use crate::bot::Bot;
use crate::listener::Listener;
use crate::opts::{Opts, ServiceOpts};

mod bot;
mod listener;
mod opts;

#[async_std::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let _guard = rusty_shared_tracing::init(opts.sentry).unwrap();

    if let Err(error) = run(opts.redis, opts.heartbeat, opts.service).await {
        error!("fatal error: {:#}", error);
    }
}

async fn run(
    redis_opts: rusty_shared_opts::redis::Opts,
    heartbeat_opts: rusty_shared_opts::heartbeat::Opts,
    service_opts: ServiceOpts,
) -> Result<()> {
    let bot_api = BotApi::new(service_opts.bot_token, std::time::Duration::from_secs(5))?;
    let me = methods::GetMe.call(&bot_api).await?;
    let redis =
        rusty_shared_redis::Redis::connect(&redis_opts.addresses, redis_opts.service_name).await?;

    let tracker_id = service_opts.tracker_id.to_lowercase();
    let bot = { Bot::new(redis.clone().await?, bot_api.clone(), me.id) };
    let listener = {
        Listener::new(
            redis,
            bot_api,
            heartbeat_opts.get_heartbeat()?,
            me.id,
            &tracker_id,
            service_opts.chat_id,
            service_opts.battery,
        )
        .await?
    };
    try_join(bot.run(), listener.run()).await?;
    Ok(())
}
