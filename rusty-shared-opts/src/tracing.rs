use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    /// Enable native logging to journald.
    #[clap(long = "enable-journald", env = "RUSTY_HOME_ENABLE_JOURNALD")]
    pub enable_journald: bool,
}
