use anyhow::Result;
use clap::Parser;
use tracing::error;

use crate::api::Api;
use crate::opts::Opts;
use crate::service::Service;

mod api;
mod models;
mod opts;
mod service;

#[async_std::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let _guard = opts.sentry.init();
    rusty_shared_tracing::init(opts.tracing.enable_journald).unwrap();

    if let Err(error) = run(opts).await {
        error!("fatal error: {:#}", error);
    }
}

async fn run(opts: Opts) -> Result<()> {
    let redis =
        rusty_shared_redis::Redis::connect(&opts.redis.addresses, opts.redis.service_name).await?;

    let service = Service::new(
        Api::new()?,
        redis,
        opts.heartbeat.get_heartbeat()?,
        opts.service,
    );
    service.run().await
}
