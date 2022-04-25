use anyhow::Result;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init() -> Result<()> {
    tracing_subscriber::Registry::default()
        .with(sentry::integrations::tracing::layer())
        .with(EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?)
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();
    Ok(())
}
