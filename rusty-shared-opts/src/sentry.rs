use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    /// Sentry SDN.
    #[clap(long = "sentry-dsn", env = "RUSTY_HOME_SENTRY_DSN")]
    pub dsn: Option<String>,

    /// Performance monitoring sample rate.
    #[clap(
        long = "sentry-traces-sample-rate",
        env = "RUSTY_HOME_SENTRY_TRACES_SAMPLE_RATE",
        default_value = "1.0"
    )]
    pub traces_sample_rate: f32,
}
