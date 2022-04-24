use anyhow::Result;
use fred::prelude::*;
use futures::TryStreamExt;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument};

use crate::models::{
    HandshakeMessage, Hardware, KeepAliveMessage, Message, Position, Token, TrackerStatusMessage,
};
use crate::Api;

pub struct Microservice {
    pub api: Api,
    pub redis: RedisClient,
    pub email: String,
    pub password: String,
}

impl Microservice {
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
            .try_filter_map(|message| async move { Ok(Some(self.handle_message(message).await?)) })
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
    async fn store_access_token(&self, key: &str, token: &Token) -> Result<()> {
        let values = HashMap::from([
            ("user_id", &token.user_id),
            ("access_token", &token.access_token),
        ]);
        let transaction = self.redis.multi(true).await?;
        transaction.hset(key, values).await?;
        transaction
            .expire_at(key, token.expires_at.timestamp())
            .await?;
        transaction.exec().await?;
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn handle_message(&self, message: Message) -> Result<()> {
        match message {
            Message::Handshake(payload) => self.handle_handshake(payload).await,
            Message::KeepAlive(payload) => self.handle_keep_alive(payload).await,
            Message::TrackerStatus(payload) => self.handle_tracker_status(payload).await,
            _ => Ok(()),
        }
    }

    #[instrument(level = "info", skip_all)]
    async fn handle_handshake(&self, payload: HandshakeMessage) -> Result<()> {
        info!(payload.keep_alive_ttl = %payload.keep_alive_ttl, "🐈 meow!");
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn handle_keep_alive(&self, payload: KeepAliveMessage) -> Result<()> {
        debug!(
            timestamp = payload.timestamp.to_string().as_str(),
            "🐈 purr…",
        );
        Ok(())
    }

    #[instrument(level = "info", skip_all, fields(payload.tracker_id = payload.tracker_id.as_str()))]
    async fn handle_tracker_status(&self, payload: TrackerStatusMessage) -> Result<()> {
        if let Some(hardware) = payload.hardware {
            self.handle_hardware_update(&payload.tracker_id, hardware)
                .await?;
        }
        if let Some(position) = payload.position {
            self.on_position_update(&payload.tracker_id, position)
                .await?;
        }
        Ok(())
    }

    #[instrument(level = "info", skip_all, fields(tracker_id = tracker_id))]
    async fn handle_hardware_update(&self, tracker_id: &str, hardware: Hardware) -> Result<()> {
        info!(
            hardware.battery_level = hardware.battery_level,
            "{}",
            if hardware.battery_level >= 50 {
                "🔋"
            } else {
                "🪫"
            },
        );
        Ok(())
    }

    #[instrument(level = "info", skip_all, fields(tracker_id = tracker_id))]
    async fn on_position_update(&self, tracker_id: &str, position: Position) -> Result<()> {
        let (latitude, longitude) = position.latlong;
        info!(
            position.latitude = latitude,
            position.longitude = longitude,
            position.accuracy = position.accuracy,
            position.timestamp = position.timestamp.to_string().as_str(),
            "🎯",
        );
        Ok(())
    }
}
