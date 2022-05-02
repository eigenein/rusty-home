use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, instrument};

use crate::{methods, models};

const USER_AGENT: &str = concat!(
    "rusty-shared-telegram/",
    env!("VERGEN_GIT_SHA_SHORT"),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

#[derive(Clone)]
pub struct BotApi {
    client: Client,
    token: String,
    timeout: std::time::Duration,
}

impl BotApi {
    #[instrument(level = "debug", skip_all)]
    pub fn new(token: String, timeout: std::time::Duration) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(timeout)
            .build()?;
        Ok(Self {
            client,
            token,
            timeout,
        })
    }

    /// https://core.telegram.org/bots/api#getme
    #[instrument(level = "info", skip_all)]
    pub async fn get_me(&self) -> Result<models::User> {
        self.call("getMe", &()).await
    }

    #[instrument(level = "debug", skip_all, fields(offset = payload.offset))]
    pub async fn get_updates(&self, payload: methods::GetUpdates) -> Result<Vec<models::Update>> {
        debug!(timeout = ?payload.timeout, "starting the long polling request…");
        let body = self
            .client
            .post(format!(
                "https://api.telegram.org/bot{}/getUpdates",
                self.token,
            ))
            .json(&payload)
            .timeout(self.timeout + payload.timeout)
            .send()
            .await
            .context("failed to send the request")?
            .bytes()
            .await?;

        debug!(body = ?body, "completed the long polling request");
        serde_json::from_slice::<models::Response<Vec<models::Update>>>(&body)
            .context("failed to deserialize response")?
            .into()
    }

    /// https://core.telegram.org/bots/api#setmycommands
    #[instrument(level = "info", skip_all)]
    pub async fn set_my_commands(&self, payload: methods::SetMyCommands) -> Result<bool> {
        self.call("setMyCommands", &payload).await
    }

    /// https://core.telegram.org/bots/api#sendmessage
    #[instrument(level = "info", skip_all, fields(chat_id = ?payload.chat_id))]
    pub async fn send_message(&self, payload: methods::SendMessage) -> Result<models::Message> {
        self.call("sendMessage", &payload).await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?chat_id, message_id = message_id))]
    pub async fn delete_message(&self, chat_id: models::ChatId, message_id: i64) -> Result<bool> {
        self.call(
            "deleteMessage",
            &methods::DeleteMessage {
                chat_id,
                message_id,
            },
        )
        .await
    }

    /// https://core.telegram.org/bots/api#sendlocation
    #[instrument(
        level = "info",
        skip_all,
        fields(
            chat_id = ?payload.location.chat_id,
            horizontal_accuracy = payload.location.horizontal_accuracy,
        ),
    )]
    pub async fn send_location(&self, payload: methods::SendLocation) -> Result<models::Message> {
        self.call("sendLocation", &payload).await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?payload.chat_id, message_id = payload.message_id))]
    pub async fn edit_message_live_location(
        &self,
        payload: methods::EditMessageLiveLocation,
    ) -> Result<models::Message> {
        self.call("editMessageLiveLocation", &payload).await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?chat_id, message_id = message_id))]
    pub async fn stop_message_live_location(
        &self,
        chat_id: models::ChatId,
        message_id: i64,
    ) -> Result<models::Message> {
        self.call(
            "stopMessageLiveLocation",
            &methods::StopMessageLiveLocation {
                chat_id,
                message_id,
            },
        )
        .await
    }

    #[instrument(level = "debug", skip_all, fields(method_name = method_name))]
    async fn call<R: DeserializeOwned>(
        &self,
        method_name: &str,
        body: &impl Serialize,
    ) -> Result<R> {
        let body = self
            .client
            .post(format!(
                "https://api.telegram.org/bot{}/{}",
                self.token, method_name,
            ))
            .json(body)
            .send()
            .await
            .with_context(|| format!("failed to send the `{}` request", method_name))?
            .bytes()
            .await?;
        debug!(body = ?body, "completed the request");
        serde_json::from_slice::<models::Response<R>>(&body)
            .with_context(|| format!("failed to deserialize `{}` response", method_name))?
            .into()
    }
}
