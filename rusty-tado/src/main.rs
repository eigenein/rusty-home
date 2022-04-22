use anyhow::Result;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::Registry::default()
        .with(sentry::integrations::tracing::layer())
        .with(EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?)
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("hello");
    Ok(())
}
