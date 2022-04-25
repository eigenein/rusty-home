use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::Parser;
use fred::clients::RedisClient;
use fred::interfaces::ClientLike;
use fred::types::{ReconnectPolicy, RedisConfig, ServerConfig};
use tracing::{info, instrument};

#[derive(Parser)]
pub struct Opts {
    /// Redis host for a centralized configuration
    #[clap(long = "redis-host", env = "RUSTY_HOME_REDIS_HOST", conflicts_with_all = &["service-name", "sentinels"])]
    host: Option<SocketAddr>,

    /// Redis Sentinel host(s) for a sentinel configuration
    #[clap(
        long = "redis-sentinel",
        env = "RUSTY_HOME_REDIS_SENTINELS",
        conflicts_with = "host",
        requires = "service-name",
        use_value_delimiter = true
    )]
    sentinels: Option<Vec<SocketAddr>>,

    /// Redis Sentinel master name, e.g. `mymaster`
    #[clap(
        long = "redis-service-name",
        env = "RUSTY_HOME_REDIS_SERVICE_NAME",
        requires = "sentinels"
    )]
    service_name: Option<String>,
}

impl Opts {
    #[instrument(level = "info", skip_all)]
    pub async fn connect(self) -> Result<RedisClient> {
        let config = RedisConfig {
            server: match self.service_name {
                None => ServerConfig::Centralized {
                    host: self.host.unwrap().ip().to_string(),
                    port: self.host.unwrap().port(),
                },
                Some(service_name) => ServerConfig::Sentinel {
                    service_name,
                    hosts: self
                        .sentinels
                        .unwrap()
                        .into_iter()
                        .map(|address| (address.ip().to_string(), address.port()))
                        .collect(),
                },
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
