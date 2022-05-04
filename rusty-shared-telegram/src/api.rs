use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, instrument};

use crate::methods::Method;
use crate::{methods, models};

const USER_AGENT: &str = concat!(
    "rusty-shared-telegram/",
    env!("VERGEN_GIT_SHA_SHORT"),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

#[derive(Clone)]
pub struct BotApi {
    client: Client,
    base_url: String,
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
            base_url: format!("https://api.telegram.org/bot{}", token),
            timeout,
        })
    }

    #[instrument(level = "debug", skip_all, fields(method = ?method))]
    pub async fn call<M: Method>(&self, method: &M) -> Result<M::Output> {
        let text = self
            .client
            .post(format!("{}/{}", self.base_url, M::NAME))
            .json(&method)
            .send()
            .await
            .with_context(|| format!("failed to send the `{}` request", M::NAME))?
            .text_with_charset("utf-8")
            .await?;
        debug!(text = ?text, "completed the request");
        serde_json::from_str::<models::Response<M::Output>>(&text)
            .with_context(|| format!("failed to deserialize `{}` response", M::NAME))?
            .into()
    }

    /// Needs to be implemented separately because of the timeout requirement.
    /// TODO: find out how to make this work with [`Self::call`].
    #[instrument(level = "debug", skip_all, fields(offset = payload.offset))]
    pub async fn get_updates(
        &self,
        payload: methods::GetUpdates,
    ) -> Result<<methods::GetUpdates as Method>::Output> {
        debug!(timeout = ?payload.timeout, "starting the long polling request…");
        let text = self
            .client
            .post(format!("{}/{}", self.base_url, methods::GetUpdates::NAME))
            .json(&payload)
            .timeout(self.timeout + payload.timeout)
            .send()
            .await
            .context("failed to send the request")?
            .text_with_charset("utf-8")
            .await?;

        debug!(text = ?text, "completed the long polling request");
        serde_json::from_str::<models::Response<Vec<models::Update>>>(&text)
            .context("failed to deserialize response")?
            .into()
    }

    /// https://core.telegram.org/bots/api#setmycommands
    #[instrument(level = "info", skip_all)]
    pub async fn set_my_commands(
        &self,
        payload: methods::SetMyCommands,
    ) -> Result<<methods::SetMyCommands as Method>::Output> {
        self.call_legacy("setMyCommands", &payload).await
    }

    /// https://core.telegram.org/bots/api#sendmessage
    #[instrument(level = "info", skip_all, fields(chat_id = ?payload.chat_id))]
    pub async fn send_message(&self, payload: methods::SendMessage) -> Result<models::Message> {
        self.call_legacy("sendMessage", &payload).await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?chat_id, message_id = message_id))]
    pub async fn delete_message(&self, chat_id: models::ChatId, message_id: i64) -> Result<bool> {
        self.call_legacy(
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
        self.call_legacy("sendLocation", &payload).await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?payload.chat_id, message_id = payload.message_id))]
    pub async fn edit_message_live_location(
        &self,
        payload: methods::EditMessageLiveLocation,
    ) -> Result<models::Message> {
        self.call_legacy("editMessageLiveLocation", &payload).await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?chat_id, message_id = message_id))]
    pub async fn stop_message_live_location(
        &self,
        chat_id: models::ChatId,
        message_id: i64,
    ) -> Result<models::Message> {
        self.call_legacy(
            "stopMessageLiveLocation",
            &methods::StopMessageLiveLocation {
                chat_id,
                message_id,
            },
        )
        .await
    }

    #[instrument(level = "info", skip_all, fields(chat_id = ?chat_id, message_id = message_id))]
    pub async fn pin_chat_message(
        &self,
        chat_id: models::ChatId,
        message_id: i64,
        disable_notification: bool,
    ) -> Result<bool> {
        let payload = methods::PinChatMessage {
            chat_id,
            message_id,
            disable_notification,
        };
        self.call_legacy("pinChatMessage", &payload).await
    }

    #[instrument(level = "debug", skip_all, fields(method_name = method_name))]
    async fn call_legacy<R: DeserializeOwned>(
        &self,
        method_name: &str,
        payload: &impl Serialize,
    ) -> Result<R> {
        let text = self
            .client
            .post(format!("{}/{}", self.base_url, method_name))
            .json(payload)
            .send()
            .await
            .with_context(|| format!("failed to send the `{}` request", method_name))?
            .text_with_charset("utf-8")
            .await?;
        debug!(text = ?text, "completed the request");
        serde_json::from_str::<models::Response<R>>(&text)
            .with_context(|| format!("failed to deserialize `{}` response", method_name))?
            .into()
    }
}
