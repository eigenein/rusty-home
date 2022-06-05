use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    /// Redis URL.
    /// See: https://docs.rs/fred/5.0.0/fred/types/struct.RedisConfig.html#method.from_url.
    #[clap(
        long = "redis-url",
        env = "RUSTY_HOME_REDIS_URL",
        default_value = "redis://localhost/0"
    )]
    pub redis_url: String,
}
