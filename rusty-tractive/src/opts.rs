use clap::{crate_version, Parser};
use rusty_shared_opts::{heartbeat, redis, sentry};

#[derive(Parser)]
#[clap(author, version = concat!(crate_version!(), "-", env!("VERGEN_GIT_SHA_SHORT")), about)]
pub struct Opts {
    #[clap(flatten)]
    pub redis: redis::Opts,

    #[clap(flatten)]
    pub sentry: sentry::Opts,

    #[clap(flatten)]
    pub heartbeat: heartbeat::Opts,

    #[clap(flatten)]
    pub service: ServiceOpts,
}

#[derive(Parser)]
pub struct ServiceOpts {
    /// Tractive account email
    #[clap(long, env = "RUSTY_TRACTIVE_EMAIL")]
    pub email: String,

    /// Tractive account password
    #[clap(long, env = "RUSTY_TRACTIVE_PASSWORD")]
    pub password: String,
}
