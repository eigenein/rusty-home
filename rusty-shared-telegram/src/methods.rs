use serde::Serialize;
use serde_with::{serde_as, DurationSeconds};

use crate::models;

#[serde_as]
#[derive(Debug, Serialize)]
pub struct GetUpdates {
    pub offset: u64,

    #[serde_as(as = "DurationSeconds<u64>")]
    pub timeout: std::time::Duration,
}

impl GetUpdates {
    pub fn new(timeout: std::time::Duration) -> Self {
        Self { timeout, offset: 0 }
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }
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

#[derive(Debug, Default, Serialize)]
pub struct SetMyCommands {
    pub commands: Vec<models::BotCommand>,
}

impl SetMyCommands {
    pub fn command(mut self, command: models::BotCommand) -> Self {
        self.commands.push(command);
        self
    }
}
