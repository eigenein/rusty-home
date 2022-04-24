use clap::Parser;

use rusty_shared_opts::redis;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Opts {
    #[clap(flatten)]
    pub redis: redis::Opts,
}
