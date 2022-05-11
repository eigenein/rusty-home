use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    /// One address is treated as Redis Server address, multiple addresses – as Sentinel addresses.
    #[clap(
        long = "redis-server",
        env = "RUSTY_HOME_REDIS_ADDRESSES",
        default_value = "127.0.0.1:6379",
        use_value_delimiter = true
    )]
    pub addresses: Vec<SocketAddr>,

    /// Redis Sentinel master name.
    #[clap(
        long = "redis-service-name",
        env = "RUSTY_HOME_REDIS_SERVICE_NAME",
        default_value = "mymaster"
    )]
    pub service_name: String,
}
