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

    #[clap(flatten)]
    pub tracing: rusty_shared_opts::tracing::Opts,

    /// Telegram Bot API token
    #[clap(long, env = "RUSTY_TELEGRAM_BOT_TOKEN")]
    pub bot_token: String,

    /// Tractive tracker ID (case-insensitive)
    #[clap(long, env = "RUSTY_TRACTIVE_TRACKER_ID")]
    pub tracker_id: String,

    /// Target chat to which the updates will be posted
    #[clap(long, env = "RUSTY_TRACTIVE_CHAT_ID")]
    pub chat_id: i64,

    #[clap(flatten)]
    pub battery: BatteryOpts,
}

#[derive(Parser)]
pub struct BatteryOpts {
    /// Minimum battery level which is treated as full
    #[clap(
        long = "battery-full-level",
        env = "RUSTY_TRACTIVE_BATTERY_FULL",
        default_value = "95"
    )]
    pub full_level: u8,

    /// Maximum battery level which is treated as low
    #[clap(
        long = "battery-low-level",
        env = "RUSTY_TRACTIVE_BATTERY_LOW",
        default_value = "50"
    )]
    pub low_level: u8,

    /// Maximum battery level which is treated as critically low
    #[clap(
        long = "battery-critical-level",
        env = "RUSTY_TRACTIVE_BATTERY_CRITICAL",
        default_value = "15"
    )]
    pub critical_level: u8,
}
