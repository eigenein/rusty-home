mod models;

use anyhow::Result;

#[async_std::main]
async fn main() -> Result<()> {
    rusty_shared_tracing::init()?;
    Ok(())
}
