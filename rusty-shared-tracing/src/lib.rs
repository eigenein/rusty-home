use anyhow::Result;
use sentry::integrations::tracing::EventFilter;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

// FIXME: remove `app_name`, figure it out automatically.
pub fn init(app_name: &str, enable_journald: bool) -> Result<()> {
    sentry::configure_scope(|scope| {
        scope.set_tag("app.name", app_name);
    });

    let sentry_layer = sentry::integrations::tracing::layer()
        .event_filter(|metadata| match metadata.level() {
            &Level::ERROR | &Level::WARN => EventFilter::Event,
            &Level::INFO | &Level::DEBUG => EventFilter::Breadcrumb,
            _ => EventFilter::Ignore,
        })
        .span_filter(|metadata| {
            matches!(
                metadata.level(),
                &Level::ERROR | &Level::WARN | &Level::INFO | &Level::DEBUG
            )
        });
    let format_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_filter(EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?);

    let registry = tracing_subscriber::Registry::default()
        .with(sentry_layer)
        .with(format_layer);
    if enable_journald {
        let journald_layer = tracing_journald::layer()?.with_field_prefix(None);
        registry.with(journald_layer).init();
    } else {
        registry.init();
    };

    Ok(())
}
