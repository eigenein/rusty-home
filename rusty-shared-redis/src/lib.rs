use std::net::SocketAddr;

use anyhow::{bail, Context, Result};
use fred::prelude::*;
use fred::types::XID;
use tracing::{debug, info, instrument};

pub struct Redis {
    pub client: RedisClient,
}

impl Redis {
    #[instrument(level = "info", skip_all, fields(n_addresses = addresses.len()))]
    pub async fn connect(addresses: &[SocketAddr], service_name: String) -> Result<Self> {
        let client = RedisClient::new(Self::new_configuration(addresses, service_name)?);
        client.connect(Some(Default::default()));
        debug!("awaiting connection…");
        client
            .wait_for_connect()
            .await
            .context("failed to connect to Redis")?;
        debug!("connected to Redis");
        Self::load_scripts(&client).await?;
        let this = Self { client };
        Ok(this)
    }

    fn new_configuration(addresses: &[SocketAddr], service_name: String) -> Result<RedisConfig> {
        let config = RedisConfig {
            server: match addresses.len() {
                0 => {
                    bail!("at least one address is required");
                }
                1 => {
                    info!("assuming centralized configuration");
                    ServerConfig::Centralized {
                        host: addresses[0].ip().to_string(),
                        port: addresses[0].port(),
                    }
                }
                _ => {
                    info!(service_name = ?service_name, "assuming Sentinel configuration");
                    ServerConfig::Sentinel {
                        service_name,
                        hosts: addresses
                            .iter()
                            .map(|address| (address.ip().to_string(), address.port()))
                            .collect(),
                    }
                }
            },
            blocking: Blocking::Error,
            ..Default::default()
        };
        Ok(config)
    }

    #[instrument(level = "debug", skip_all)]
    async fn load_scripts(client: &RedisClient) -> Result<()> {
        let set_if_greater_sha = client
            .script_load(
                // language=lua
                r#"
                local new_value = tonumber(ARGV[1]);
                local current_value = redis.call('GET', KEYS[1]);
                if current_value == false or tonumber(current_value) < new_value
                then
                    return true
                end"#,
            )
            .await
            .context("failed to load the script «set if greater»")?;
        debug!(set_if_greater_sha = ?set_if_greater_sha);
        Ok(())
    }
}

/// Replaces an unknown error with [`Result::Ok`].
///
/// This is needed, for example, to safely insert duplicate entries into a stream.
pub fn ignore_unknown_error(error: RedisError) -> Result<(), RedisError> {
    if error.kind() == &RedisErrorKind::Unknown {
        debug!("ignoring error: {:#}", error);
        Ok(())
    } else {
        Err(error)
    }
}

#[instrument(level = "info", skip_all, fields(key = key, group_name = group_name))]
pub async fn create_consumer_group(
    redis: &RedisClient,
    key: &str,
    group_name: &str,
) -> Result<(), RedisError> {
    redis
        .xgroup_create(key, group_name, XID::Max, true)
        .await
        .or_else(|error| {
            if error.details().starts_with("BUSYGROUP") {
                Ok(())
            } else {
                Err(error)
            }
        })
}
