use std::borrow::Cow;

use anyhow::Result;
use sentry::integrations::tracing::EventFilter;
use sentry::{ClientInitGuard, ClientOptions};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub fn init(
    tracing_opts: rusty_shared_opts::tracing::Opts,
    sentry_opts: rusty_shared_opts::sentry::Opts,
) -> Result<ClientInitGuard> {
    let guard = sentry::init((
        sentry_opts.dsn,
        ClientOptions {
            release: Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
            traces_sample_rate: sentry_opts.traces_sample_rate,
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_tag("app.name", env!("CARGO_CRATE_NAME"));
    });

    let sentry_layer = sentry::integrations::tracing::layer()
        .event_filter(|metadata| match metadata.level() {
            &Level::ERROR | &Level::WARN => EventFilter::Event,
            &Level::INFO | &Level::DEBUG | &Level::TRACE => EventFilter::Breadcrumb,
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
        .with(if tracing_opts.enable_journald {
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

    Ok(guard)
}
