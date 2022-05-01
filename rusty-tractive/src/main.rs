use anyhow::Result;
use clap::Parser;

use crate::api::Api;
use crate::microservice::Microservice;
use crate::opts::Opts;

mod api;
mod microservice;
mod models;
mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = opts.sentry.init();
    rusty_shared_tracing::init()?;

    let redis = rusty_shared_redis::connect(&opts.redis.addresses, opts.redis.service_name).await?;

    let microservice = Microservice::new(
        Api::new()?,
        redis,
        opts.heartbeat.get_heartbeat()?,
        opts.email.to_lowercase(),
        opts.password,
    );
    microservice.run().await
}
