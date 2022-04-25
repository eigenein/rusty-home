use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::Parser;
use fred::clients::RedisClient;
use fred::interfaces::ClientLike;
use fred::types::{ReconnectPolicy, RedisConfig, ServerConfig};
use tracing::{info, instrument};

#[derive(Parser)]
pub struct Opts {
    /// One address is treated as Redis Server address, multiple addresses – as Sentinel addresses
    #[clap(
        long = "redis-server",
        env = "RUSTY_HOME_REDIS_ADDRESSES",
        default_value = "127.0.0.1:6379",
        use_value_delimiter = true
    )]
    addresses: Vec<SocketAddr>,

    /// Redis Sentinel master name
    #[clap(
        long = "redis-service-name",
        env = "RUSTY_HOME_REDIS_SERVICE_NAME",
        default_value = "mymaster"
    )]
    service_name: String,
}

impl Opts {
    #[instrument(
        level = "info",
        skip_all,
        fields(n_addresses = self.addresses.len(), service_name = self.service_name.as_str()),
    )]
    pub async fn connect(self) -> Result<RedisClient> {
        let config = RedisConfig {
            server: if self.addresses.len() == 1 {
                info!("assuming centralized configuration");
                ServerConfig::Centralized {
                    host: self.addresses[0].ip().to_string(),
                    port: self.addresses[0].port(),
                }
            } else {
                info!("assuming Sentinel configuration");
                ServerConfig::Sentinel {
                    service_name: self.service_name,
                    hosts: self
                        .addresses
                        .into_iter()
                        .map(|address| (address.ip().to_string(), address.port()))
                        .collect(),
                }
            },
            ..Default::default()
        };
        let policy = ReconnectPolicy::default();
        let client = RedisClient::new(config);

        client.connect(Some(policy));
        info!("awaiting connection…");
        client
            .wait_for_connect()
            .await
            .context("failed to connect to Redis")?;
        info!("connected to Redis");

        Ok(client)
    }
}
