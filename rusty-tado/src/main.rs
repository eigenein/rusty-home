use anyhow::Result;
use tracing::info;

#[async_std::main]
async fn main() -> Result<()> {
    rusty_shared_tracing::init(env!("CARGO_BIN_NAME"), false)?; // TODO
    info!("hello");
    Ok(())
}
