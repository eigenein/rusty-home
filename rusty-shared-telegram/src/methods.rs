use std::fmt::Debug;
use std::time;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_with::{serde_as, DurationSeconds};
use tracing::{debug, info, instrument};

use crate::api::BotApi;
use crate::models;

#[async_trait]
pub trait Method: Debug + Sized + Serialize {
    type Output: DeserializeOwned;

    /// The method's name in Telegram Bot API.
    const NAME: &'static str;

    /// Call the method on the specified connection.
    #[instrument(skip_all, fields(name = Self::NAME))]
    async fn call(&self, api: &BotApi) -> Result<Self::Output> {
        info!("calling…");
        debug!(self = ?self);
        let text = api
            .client
            .post(format!("{}/{}", api.base_url, Self::NAME))
            .json(self)
            .send()
            .await
            .with_context(|| format!("failed to send the `{}` request", Self::NAME))?
            .text_with_charset("utf-8")
            .await?;

        debug!(response.text = ?text, "completed the request");
        serde_json::from_str::<models::Response<Self::Output>>(&text)
            .with_context(|| format!("failed to deserialize `{}` response", Self::NAME))?
            .into()
    }
}

/// https://core.telegram.org/bots/api#getme
#[derive(Debug, Serialize)]
pub struct GetMe;

impl Method for GetMe {
    type Output = models::User;

    const NAME: &'static str = "getMe";
}

/// https://core.telegram.org/bots/api#setwebhook
#[derive(Debug, Serialize, Default)]
pub struct SetWebhook<'a> {
    pub url: String,
    pub allowed_updates: Vec<AllowedUpdate>,
    pub secret_token: Option<&'a str>,
}

impl<'a> Method for SetWebhook<'a> {
    type Output = bool;

    const NAME: &'static str = "setWebhook";
}

impl<'a> SetWebhook<'a> {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    pub fn allow_update(mut self, allowed_update: AllowedUpdate) -> Self {
        self.allowed_updates.push(allowed_update);
        self
    }

    pub const fn secret_token(mut self, secret_token: &'a str) -> Self {
        self.secret_token = Some(secret_token);
        self
    }
}

#[derive(Debug, Serialize)]
pub enum AllowedUpdate {
    #[serde(rename = "message")]
    Message,
}

/// https://core.telegram.org/bots/api#sendmessage
#[derive(Debug, Serialize)]
pub struct SendMessage {
    pub chat_id: models::ChatId,
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<models::ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,
}

impl Method for SendMessage {
    type Output = models::Message;

    const NAME: &'static str = "sendMessage";
}

impl SendMessage {
    pub fn new(chat_id: impl Into<models::ChatId>, text: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            text: text.into(),
            parse_mode: None,
            reply_to_message_id: None,
        }
    }

    pub const fn parse_mode(mut self, parse_mode: models::ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub const fn reply_to_message_id(mut self, reply_to_message_id: i64) -> Self {
        self.reply_to_message_id = Some(reply_to_message_id);
        self
    }
}

/// https://core.telegram.org/bots/api#setmycommands
#[derive(Debug, Default, Serialize)]
pub struct SetMyCommands {
    pub commands: Vec<models::BotCommand>,
}

impl Method for SetMyCommands {
    type Output = bool;

    const NAME: &'static str = "setMyCommands";
}

impl SetMyCommands {
    pub fn command(mut self, command: models::BotCommand) -> Self {
        self.commands.push(command);
        self
    }
}

/// Shared location parameters.
#[derive(Debug, Serialize)]
pub struct Location {
    pub chat_id: models::ChatId,
    pub latitude: f64,
    pub longitude: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f32>,
}

impl Location {
    pub const fn new(chat_id: models::ChatId, latitude: f64, longitude: f64) -> Self {
        Self {
            chat_id,
            latitude,
            longitude,
            heading: None,
            horizontal_accuracy: None,
        }
    }

    pub const fn horizontal_accuracy(mut self, horizontal_accuracy: f32) -> Self {
        self.horizontal_accuracy = Some(horizontal_accuracy);
        self
    }

    pub fn heading<H: Into<Option<u16>>>(mut self, heading: H) -> Self {
        self.heading = heading.into();
        self
    }
}

/// https://core.telegram.org/bots/api#sendlocation
#[serde_as]
#[derive(Debug, Serialize)]
pub struct SendLocation {
    #[serde(flatten)]
    pub location: Location,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub live_period: Option<time::Duration>,
}

impl Method for SendLocation {
    type Output = models::Message;

    const NAME: &'static str = "sendLocation";
}

impl SendLocation {
    pub const fn new(location: Location) -> Self {
        Self {
            location,
            live_period: None,
        }
    }

    pub const fn live_period(mut self, live_period: time::Duration) -> Self {
        self.live_period = Some(live_period);
        self
    }
}

/// https://core.telegram.org/bots/api#editmessagelivelocation
#[derive(Debug, Serialize)]
pub struct EditMessageLiveLocation {
    pub chat_id: models::ChatId,

    pub message_id: i64,

    #[serde(flatten)]
    pub location: Location,
}

impl Method for EditMessageLiveLocation {
    type Output = models::Message;

    const NAME: &'static str = "editMessageLiveLocation";
}

impl EditMessageLiveLocation {
    pub const fn new(chat_id: models::ChatId, message_id: i64, location: Location) -> Self {
        Self {
            chat_id,
            message_id,
            location,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteMessage {
    pub chat_id: models::ChatId,
    pub message_id: i64,
}

impl Method for DeleteMessage {
    type Output = bool;

    const NAME: &'static str = "deleteMessage";
}

impl DeleteMessage {
    pub fn new(chat_id: impl Into<models::ChatId>, message_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StopMessageLiveLocation {
    pub chat_id: models::ChatId,
    pub message_id: i64,
}

impl Method for StopMessageLiveLocation {
    type Output = models::Message;

    const NAME: &'static str = "stopMessageLiveLocation";
}

#[derive(Debug, Serialize)]
pub struct PinChatMessage {
    pub chat_id: models::ChatId,
    pub message_id: i64,
    pub disable_notification: bool,
}

impl Method for PinChatMessage {
    type Output = bool;

    const NAME: &'static str = "pinChatMessage";
}

impl PinChatMessage {
    pub fn new(chat_id: impl Into<models::ChatId>, message_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
            disable_notification: false,
        }
    }

    pub const fn disable_notification(mut self) -> Self {
        self.disable_notification = true;
        self
    }
}

#[must_use]
#[derive(Debug, Serialize)]
pub struct UnpinChatMessage {
    pub chat_id: models::ChatId,
    pub message_id: i64,
}

impl Method for UnpinChatMessage {
    type Output = bool;

    const NAME: &'static str = "unpinChatMessage";
}

impl UnpinChatMessage {
    pub fn new(chat_id: impl Into<models::ChatId>, message_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
        }
    }
}
