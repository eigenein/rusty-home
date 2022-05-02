use anyhow::Result;
use clap::Parser;
use sentry::integrations::anyhow::capture_anyhow;

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
    rusty_shared_tracing::init().unwrap();

    if let Err(error) = run(opts).await {
        capture_anyhow(&error);
    }
}

async fn run(opts: Opts) -> Result<()> {
    let redis = rusty_shared_redis::connect(&opts.redis.addresses, opts.redis.service_name).await?;

    let microservice = Service::new(
        Api::new()?,
        redis,
        opts.heartbeat.get_heartbeat()?,
        opts.email.to_lowercase(),
        opts.password,
    );
    microservice.run().await
}
