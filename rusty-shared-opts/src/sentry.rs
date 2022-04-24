use clap::Parser;
use sentry::{ClientInitGuard, ClientOptions};

#[derive(Parser)]
pub struct Opts {
    /// Sentry SDN
    #[clap(long = "sentry-dsn", env = "RUSTY_HOME_SENTRY_DSN")]
    pub dsn: Option<String>,

    /// Performance monitoring sample rate
    #[clap(
        long = "sentry-traces-sample-rate",
        env = "RUSTY_HOME_SENTRY_TRACES_SAMPLE_RATE",
        default_value = "1.0"
    )]
    pub traces_sample_rate: f32,
}

impl Opts {
    pub fn init(self) -> ClientInitGuard {
        sentry::init((
            self.dsn,
            ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: self.traces_sample_rate,
                ..Default::default()
            },
        ))
    }
}
