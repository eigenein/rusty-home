use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use futures::future::try_join;
use rusty_shared_telegram::api::BotApi;
use rusty_shared_telegram::methods;
use rusty_shared_telegram::methods::Method;

use crate::listener::Listener;
use crate::opts::Opts;

mod bot;
mod listener;
mod middleware;
mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = rusty_shared_tracing::init(opts.sentry, env!("CARGO_BIN_NAME"))?;

    let bot_api = BotApi::new(opts.service.bot_token, Duration::from_secs(5))?;
    let me = methods::GetMe.call(&bot_api).await?;
    let redis = rusty_shared_redis::Redis::connect(&opts.redis.redis_url).await?;

    let listener = {
        let bot_api = bot_api.clone();
        let tracker_id = opts.service.tracker_id.to_lowercase();
        let chat_id = opts.service.chat_id;
        let heartbeat = opts.heartbeat.get_heartbeat()?;
        let battery_opts = opts.service.battery;
        Listener::new(redis, bot_api, heartbeat, me.id, &tracker_id, chat_id, battery_opts).await?
    };
    let listener_future = listener.run();

    let bot_future = bot::run(
        bot_api,
        opts.service.bind_endpoint,
        opts.service.webhook_url,
        opts.service.secret_token,
    );
    try_join(bot_future, listener_future).await?;
    Ok(())
}
