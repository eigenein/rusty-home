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

    let redis = opts.redis.connect().await?;

    let microservice = Microservice {
        redis,
        api: Api::new()?,
        heartbeat: opts.heartbeat.get_heartbeat()?,
        email: opts.email,
        password: opts.password,
    };
    microservice.run().await
}
