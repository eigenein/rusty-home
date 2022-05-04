use std::fmt::Debug;
use std::time;

use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_with::{serde_as, DurationSeconds};

use crate::api::BotApi;
use crate::models;

#[async_trait]
pub trait Method: Debug + Sized + Serialize {
    type Output: DeserializeOwned;

    /// The method's name in Telegram Bot API.
    const NAME: &'static str;

    /// Call the method on the specified connection.
    async fn call(&self, api: &BotApi) -> Result<Self::Output> {
        api.call(self).await
    }
}

/// https://core.telegram.org/bots/api#getme
#[derive(Debug, Serialize)]
pub struct GetMe;

impl Method for GetMe {
    type Output = models::User;

    const NAME: &'static str = "getMe";
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct GetUpdates {
    pub offset: u64,

    #[serde_as(as = "DurationSeconds<u64>")]
    pub timeout: time::Duration,

    pub allowed_updates: Vec<AllowedUpdate>,
}

impl Method for GetUpdates {
    type Output = Vec<models::Update>;

    const NAME: &'static str = "getUpdates";
}

impl GetUpdates {
    pub fn new(timeout: time::Duration) -> Self {
        Self {
            timeout,
            offset: 0,
            allowed_updates: Vec::new(),
        }
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    pub fn allowed_update(mut self, allowed_update: AllowedUpdate) -> Self {
        self.allowed_updates.push(allowed_update);
        self
    }
}

#[derive(Debug, Serialize)]
pub enum AllowedUpdate {
    #[serde(rename = "message")]
    Message,
}

#[derive(Debug, Serialize)]
pub struct SendMessage {
    pub chat_id: models::ChatId,
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<models::ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,
}

impl SendMessage {
    pub fn new(chat_id: models::ChatId, text: String) -> Self {
        Self {
            chat_id,
            text,
            parse_mode: None,
            reply_to_message_id: None,
        }
    }

    pub fn parse_mode(mut self, parse_mode: models::ParseMode) -> Self {
        self.parse_mode = Some(parse_mode);
        self
    }

    pub fn reply_to_message_id(mut self, reply_to_message_id: i64) -> Self {
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
    pub fn new(chat_id: models::ChatId, latitude: f64, longitude: f64) -> Self {
        Self {
            chat_id,
            latitude,
            longitude,
            heading: None,
            horizontal_accuracy: None,
        }
    }

    pub fn horizontal_accuracy(mut self, horizontal_accuracy: f32) -> Self {
        self.horizontal_accuracy = Some(horizontal_accuracy);
        self
    }

    pub fn heading(mut self, heading: u16) -> Self {
        self.heading = Some(heading);
        self
    }
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct SendLocation {
    #[serde(flatten)]
    pub location: Location,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub live_period: Option<time::Duration>,
}

impl SendLocation {
    pub fn new(location: Location) -> Self {
        Self {
            location,
            live_period: None,
        }
    }

    pub fn live_period(mut self, live_period: time::Duration) -> Self {
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

impl EditMessageLiveLocation {
    pub fn new(chat_id: models::ChatId, message_id: i64, location: Location) -> Self {
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

#[derive(Debug, Serialize)]
pub struct StopMessageLiveLocation {
    pub chat_id: models::ChatId,
    pub message_id: i64,
}

#[derive(Debug, Serialize)]
pub struct PinChatMessage {
    pub chat_id: models::ChatId,
    pub message_id: i64,
    pub disable_notification: bool,
}
