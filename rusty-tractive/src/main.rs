use anyhow::Result;

use crate::api::Api;

mod api;
mod models;

#[async_std::main]
async fn main() -> Result<()> {
    rusty_shared_tracing::init()?;

    // let mut api = Api::new("", "")?;
    // api.authenticate().await?;

    Ok(())
}
