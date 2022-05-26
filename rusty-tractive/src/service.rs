use std::collections::HashMap;
use std::time;

use anyhow::{bail, Context, Result};
use async_std::future::timeout;
use fred::prelude::*;
use futures::StreamExt;
use kv_derive::prelude::*;
use rusty_shared_opts::heartbeat::Heartbeat;
use rusty_shared_redis::Redis;
use rusty_shared_tractive::{hardware_stream_key, position_stream_key, HardwareEntry};
use tracing::{debug, info, instrument};

use crate::models::*;
use crate::opts::ServiceOpts;
use crate::Api;

pub struct Service {
    pub api: Api,
    pub redis: Redis,
    pub heartbeat: Heartbeat,
    pub opts: ServiceOpts,
}

impl Service {
    pub async fn run(&self) -> Result<()> {
        let (user_id, access_token) = self
            .get_authentication()
            .await
            .context("failed to authenticate")?;

        let mut messages = Box::pin(self.api.get_messages(&user_id, &access_token).await?);
        let mut keep_alive_ttl = time::Duration::from_secs(600);

        while let Some(message) = timeout(keep_alive_ttl, messages.next()).await? {
            match message? {
                Message::Handshake(payload) => {
                    info!(keep_alive_ttl = ?payload.keep_alive_ttl, "🐈 meow!");
                    keep_alive_ttl = payload.keep_alive_ttl;
                }
                Message::KeepAlive(payload) => {
                    debug!(timestamp = ?payload.timestamp, "🐈 purr…",);
                }
                Message::TrackerStatus(payload) => {
                    self.on_tracker_status(payload).await?;
                }
                _ => {}
            };
        }

        bail!("the message stream has ended unexpectedly");
    }

    #[tracing::instrument(skip_all, fields(self.email = ?self.opts.email))]
    async fn get_authentication(&self) -> Result<(String, String)> {
        let key = format!("rusty:tractive:{}:authentication", self.opts.email);
        let authentication: HashMap<String, String> = self.redis.client.hgetall(&key).await?;
        let result = match (authentication.get("user_id"), authentication.get("access_token")) {
            (Some(user_id), Some(access_token)) => {
                debug!("using the cached token");
                (user_id.clone(), access_token.clone())
            }
            _ => {
                let token = self
                    .api
                    .authenticate(&self.opts.email, &self.opts.password)
                    .await?;
                self.store_access_token(&key, &token).await?;
                (token.user_id, token.access_token)
            }
        };

        Ok(result)
    }

    #[instrument(skip_all, fields(user_id = ?token.user_id))]
    async fn store_access_token(&self, key: &str, token: &Token) -> Result<()> {
        let values = vec![
            ("user_id", &token.user_id),
            ("access_token", &token.access_token),
        ];
        let transaction = self.redis.client.multi(true).await?;
        transaction.hset(key, values).await?;
        transaction
            .expire_at(key, token.expires_at.timestamp())
            .await?;
        transaction.exec().await?;
        Ok(())
    }

    #[instrument(skip_all, fields(tracker_id = ?payload.tracker_id))]
    async fn on_tracker_status(&self, payload: TrackerStatusMessage) -> Result<()> {
        let tracker_id = payload.tracker_id.to_lowercase();
        if let Some(hardware) = payload.hardware {
            self.on_hardware_update(&tracker_id, hardware).await?;
        }
        if let Some(position) = payload.position {
            self.on_position_update(&tracker_id, position).await?;
        }
        self.heartbeat.send().await;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn on_hardware_update(&self, tracker_id: &str, hardware: HardwareEntry) -> Result<()> {
        info!(timestamp = ?hardware.timestamp, battery_level = hardware.battery_level, "⌚ hardware update️");
        let (is_timestamp_updated, _) = self
            .redis
            .set_if_greater(
                format!("rusty:tractive:{}:hardware:last_timestamp", tracker_id),
                hardware.timestamp.timestamp(),
            )
            .await
            .context("failed to update the last hardware timestamp")?;
        if !is_timestamp_updated {
            info!("⌚ timestamp is not updated");
            return Ok(());
        }
        self.redis
            .client
            .xadd(
                hardware_stream_key(tracker_id),
                false,
                None,
                format!("{}-0", hardware.timestamp.timestamp_millis()),
                hardware.into_vec(),
            )
            .await
            .context("failed to push the hardware stream entry")
    }

    #[instrument(skip_all)]
    async fn on_position_update(&self, tracker_id: &str, position: Position) -> Result<()> {
        let (latitude, longitude) = position.latlong;
        info!(
            timestamp = ?position.timestamp,
            latitude,
            longitude,
            accuracy = position.accuracy,
            course = position.course,
            "🎯 position update",
        );
        let (is_timestamp_updated, _) = self
            .redis
            .set_if_greater(
                format!("rusty:tractive:{}:position:last_timestamp", tracker_id),
                position.timestamp.timestamp(),
            )
            .await
            .context("failed to update the last position timestamp")?;
        if !is_timestamp_updated {
            info!("🎯 timestamp is not updated");
            return Ok(());
        }
        let mut fields = vec![
            ("ts", RedisValue::from(position.timestamp.timestamp())),
            ("lat", RedisValue::from(latitude)),
            ("lon", RedisValue::from(longitude)),
            ("accuracy", RedisValue::from(position.accuracy)),
        ];
        if let Some(course) = position.course {
            fields.push(("course", RedisValue::from(course)));
        }
        self.redis
            .client
            .xadd(
                position_stream_key(tracker_id),
                false,
                None,
                format!("{}-0", position.timestamp.timestamp_millis()),
                fields,
            )
            .await
            .context("failed to push the position stream entry")
    }
}
