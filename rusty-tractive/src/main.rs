use anyhow::Result;
use clap::Parser;
use tracing::error;

use crate::api::Api;
use crate::opts::{Opts, ServiceOpts};
use crate::service::Service;

mod api;
mod models;
mod opts;
mod service;

#[async_std::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let _guard = rusty_shared_tracing::init(opts.tracing, opts.sentry).unwrap();

    if let Err(error) = run(opts.redis, opts.heartbeat, opts.service).await {
        error!("fatal error: {:#}", error);
    }
}

async fn run(
    redis_opts: rusty_shared_opts::redis::Opts,
    heartbeat_opts: rusty_shared_opts::heartbeat::Opts,
    service_opts: ServiceOpts,
) -> Result<()> {
    let redis =
        rusty_shared_redis::Redis::connect(&redis_opts.addresses, redis_opts.service_name).await?;

    let service = Service {
        api: Api::new()?,
        redis,
        heartbeat: heartbeat_opts.get_heartbeat()?,
        opts: service_opts,
    };
    service.run().await
}
