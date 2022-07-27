use anyhow::Result;
use clap::Parser;

use crate::api::Api;
use crate::opts::Opts;
use crate::service::Service;

mod api;
mod models;
mod opts;
mod service;

static BIN_NAME: &str = env!("CARGO_BIN_NAME");

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = rusty_shared_tracing::init(opts.sentry, BIN_NAME)?;

    let service = Service {
        api: Api::new()?,
        redis: rusty_shared_redis::Redis::connect(&opts.redis.redis_url, BIN_NAME).await?,
        heartbeat: opts.heartbeat.get_heartbeat()?,
        opts: opts.service,
    };
    service.run().await
}
