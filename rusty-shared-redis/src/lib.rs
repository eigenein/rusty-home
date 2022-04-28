use std::net::SocketAddr;

use anyhow::{bail, Context, Result};
use fred::prelude::*;
use tracing::{debug, info, instrument};

#[instrument(
    level = "info",
    skip_all,
    fields(n_addresses = addresses.len(), service_name = service_name.as_str()),
)]
pub async fn connect(addresses: Vec<SocketAddr>, service_name: String) -> Result<RedisClient> {
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
                info!("assuming Sentinel configuration");
                ServerConfig::Sentinel {
                    service_name,
                    hosts: addresses
                        .into_iter()
                        .map(|address| (address.ip().to_string(), address.port()))
                        .collect(),
                }
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
