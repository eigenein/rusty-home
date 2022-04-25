use anyhow::Result;
use clap::Parser;

use crate::opts::Opts;

mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = opts.sentry.init();
    rusty_shared_tracing::init()?;

    let redis = opts.redis.connect().await?;

    Ok(())
}
