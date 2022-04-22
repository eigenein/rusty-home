use anyhow::Result;
use tracing::info;

#[async_std::main]
async fn main() -> Result<()> {
    rusty_shared_tracing::init()?;
    info!("hello");
    Ok(())
}
