use std::collections::HashMap;

use anyhow::{Context, Result};
use fred::prelude::*;
use futures::TryStreamExt;
use rusty_shared_opts::heartbeat::Heartbeat;
use rusty_shared_redis::ignore_unknown_error;
use tracing::{debug, error, info, instrument};

use crate::{models, Api};

pub struct Service {
    api: Api,
    redis: RedisClient,
    heartbeat: Heartbeat,
    email: String,
    password: String,
}

impl Service {
    pub fn new(
        api: Api,
        redis: RedisClient,
        heartbeat: Heartbeat,
        email: String,
        password: String,
    ) -> Self {
        Self {
            api,
            redis,
            heartbeat,
            email,
            password,
        }
    }

    pub async fn run(self) -> ! {
        loop {
            if let Err(error) = self.loop_().await {
                error!("{:#}", error);
            }
        }
    }

    async fn loop_(&self) -> Result<()> {
        let (user_id, access_token) = match self.get_authentication().await {
            Ok(access_token) => access_token,
            Err(error) => panic!("failed to obtain the access token: {:#}", error),
        };

        self.api
            .get_messages(&user_id, &access_token)
            .await?
            .try_filter_map(|message| async move { Ok(Some(self.on_message(message).await?)) })
            .try_collect()
            .await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(self.email = self.email.as_str()))]
    async fn get_authentication(&self) -> Result<(String, String)> {
        let key = format!("rusty:tractive:{}:authentication", self.email);
        let authentication: HashMap<String, String> = self.redis.hgetall(&key).await?;
        let result = match (
            authentication.get("user_id"),
            authentication.get("access_token"),
        ) {
            (Some(user_id), Some(access_token)) => {
                debug!("using the cached token");
                (user_id.clone(), access_token.clone())
            }
            _ => {
                let token = self.api.authenticate(&self.email, &self.password).await?;
                self.store_access_token(&key, &token).await?;
                (token.user_id, token.access_token)
            }
        };

        Ok(result)
    }

    #[instrument(level = "info", skip_all, fields(user_id = token.user_id.as_str()))]
    async fn store_access_token(&self, key: &str, token: &models::Token) -> Result<()> {
        let values = vec![
            ("user_id", &token.user_id),
            ("access_token", &token.access_token),
        ];
        let transaction = self.redis.multi(true).await?;
        transaction.hset(key, values).await?;
        transaction
            .expire_at(key, token.expires_at.timestamp())
            .await?;
        transaction.exec().await?;
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn on_message(&self, message: models::Message) -> Result<()> {
        let result = match message {
            models::Message::Handshake(payload) => self.on_handshake(payload).await,
            models::Message::KeepAlive(payload) => self.on_keep_alive(payload).await,
            models::Message::TrackerStatus(payload) => self.on_tracker_status(payload).await,
            _ => Ok(()),
        };
        if let Err(error) = result {
            error!("failed to handle the message: {:#}", error);
        } else {
            self.heartbeat.send().await;
        }
        Ok(())
    }

    #[instrument(level = "info", skip_all)]
    async fn on_handshake(&self, payload: models::HandshakeMessage) -> Result<()> {
        info!(keep_alive_ttl = %payload.keep_alive_ttl, "🐈 meow!");
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn on_keep_alive(&self, payload: models::KeepAliveMessage) -> Result<()> {
        debug!(
            timestamp = payload.timestamp.to_string().as_str(),
            "🐈 purr…",
        );
        Ok(())
    }

    #[instrument(level = "info", skip_all, fields(tracker_id = payload.tracker_id.as_str()))]
    async fn on_tracker_status(&self, payload: models::TrackerStatusMessage) -> Result<()> {
        let tracker_id = payload.tracker_id.to_lowercase();
        if let Some(hardware) = payload.hardware {
            self.on_hardware_update(&tracker_id, hardware).await?;
        }
        if let Some(position) = payload.position {
            self.on_position_update(&tracker_id, position).await?;
        }
        Ok(())
    }

    #[instrument(level = "info", skip_all)]
    async fn on_hardware_update(&self, tracker_id: &str, hardware: models::Hardware) -> Result<()> {
        info!(
            timestamp = hardware.timestamp.to_string().as_str(),
            battery_level = hardware.battery_level,
            "⌚️",
        );
        self.redis
            .xadd(
                format!("rusty:tractive:{}:hardware", tracker_id),
                false,
                None,
                format!("{}-0", hardware.timestamp.timestamp_millis()),
                vec![
                    ("ts", hardware.timestamp.timestamp().to_string()),
                    ("battery", hardware.battery_level.to_string()),
                ],
            )
            .await
            .or_else(ignore_unknown_error)
            .context("failed to push the hardware stream entry")
    }

    #[instrument(level = "info", skip_all)]
    async fn on_position_update(&self, tracker_id: &str, position: models::Position) -> Result<()> {
        let (latitude, longitude) = position.latlong;
        info!(
            timestamp = position.timestamp.to_string().as_str(),
            latitude = latitude,
            longitude = longitude,
            accuracy = position.accuracy,
            course = position.course,
            "🎯",
        );
        let mut fields = vec![
            ("ts", position.timestamp.timestamp().to_string()),
            ("lat", latitude.to_string()),
            ("lon", longitude.to_string()),
            ("accuracy", position.accuracy.to_string()),
        ];
        if let Some(course) = position.course {
            fields.push(("course", course.to_string()));
        }
        self.redis
            .xadd(
                format!("rusty:tractive:{}:position", tracker_id),
                false,
                None,
                format!("{}-0", position.timestamp.timestamp_millis()),
                fields,
            )
            .await
            .or_else(ignore_unknown_error)
            .context("failed to push the position stream entry")
    }
}