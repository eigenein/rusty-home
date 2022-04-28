use anyhow::Result;
use clap::Parser;
use tracing::info;

use rusty_shared_telegram::api::Api;

use crate::opts::Opts;

mod opts;

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let _guard = opts.sentry.init();
    rusty_shared_tracing::init()?;

    let _redis = rusty_shared_redis::connect(opts.redis.addresses, opts.redis.service_name).await?;
    let bot_api = Api::new(opts.bot_token)?;

    for update in bot_api
        .get_updates(std::time::Duration::from_secs(5))
        .await?
    {
        info!("{:?}", update);
    }

    Ok(())
}
