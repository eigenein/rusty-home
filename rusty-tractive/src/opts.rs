use clap::Parser;

use rusty_shared_opts::{heartbeat, redis, sentry};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Opts {
    #[clap(flatten)]
    pub redis: redis::Opts,

    #[clap(flatten)]
    pub sentry: sentry::Opts,

    #[clap(flatten)]
    pub heartbeat: heartbeat::Opts,

    /// Tractive account email
    #[clap(long, env = "RUSTY_TRACTIVE_EMAIL")]
    pub email: String,

    /// Tractive account password
    #[clap(long, env = "RUSTY_TRACTIVE_PASSWORD")]
    pub password: String,
}
