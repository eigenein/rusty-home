use anyhow::Result;
use sentry::integrations::tracing::EventFilter;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub fn init(enable_journald: bool) -> Result<()> {
    sentry::configure_scope(|scope| {
        scope.set_tag("app.name", env!("CARGO_CRATE_NAME"));
    });

    let sentry_layer = sentry::integrations::tracing::layer()
        .event_filter(|metadata| match metadata.level() {
            &Level::ERROR | &Level::WARN => EventFilter::Event,
            &Level::INFO | &Level::DEBUG => EventFilter::Breadcrumb,
            _ => EventFilter::Ignore,
        })
        .span_filter(|metadata| {
            matches!(metadata.level(), &Level::ERROR | &Level::WARN | &Level::INFO | &Level::DEBUG)
        });

    let format_filter =
        EnvFilter::try_from_env("RUSTY_HOME_LOG").or_else(|_| EnvFilter::try_new("info"))?;
    let format_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_filter(format_filter);

    tracing_subscriber::Registry::default()
        .with(sentry_layer)
        .with(format_layer)
        .with(if enable_journald {
            let filter = EnvFilter::try_from_env("RUSTY_HOME_JOURNALD")
                .or_else(|_| EnvFilter::try_new("info"))?;
            let layer = tracing_journald::layer()?
                .with_field_prefix(None)
                .with_syslog_identifier(env!("CARGO_CRATE_NAME").to_string())
                .with_filter(filter);
            Some(layer)
        } else {
            None
        })
        .init();

    Ok(())
}
