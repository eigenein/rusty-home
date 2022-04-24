use anyhow::Result;
use clap::Parser;

use crate::api::Api;
use crate::opts::Opts;

mod api;
mod models;
mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    rusty_shared_tracing::init()?;

    opts.redis.get_client().await?;

    // let mut api = Api::new("", "")?;
    // api.authenticate().await?;

    Ok(())
}
