use std::borrow::Cow;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Ok {
        result: T,
    },
    Err {
        error_code: i32,
        description: String,
    },
}

impl<T> From<Response<T>> for Result<T> {
    fn from(response: Response<T>) -> Self {
        match response {
            Response::Ok { result } => Ok(result),
            Response::Err {
                error_code,
                description,
            } => Err(anyhow!("error {}: {}", error_code, description)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct User {
    /// Unique identifier for this user or bot.
    pub id: i64,

    /// User's or bot's first name.
    pub first_name: String,

    /// User's or bot's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    /// Unique identifier for this chat.
    pub id: i64,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChatId {
    /// Unique identifier for the target chat.
    UniqueId(i64),

    /// Username of the target channel (in the format `@channelusername`).
    Username(Cow<'static, str>),
}

impl From<i64> for ChatId {
    fn from(chat_id: i64) -> Self {
        Self::UniqueId(chat_id)
    }
}

#[derive(Debug, Deserialize)]
pub struct Update {
    #[serde(rename = "update_id")]
    pub id: i64,

    #[serde(flatten)]
    pub payload: UpdatePayload,
}

#[derive(Debug, Deserialize)]
pub enum UpdatePayload {
    #[serde(rename = "message")]
    Message(Message),

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    #[serde(rename = "message_id")]
    pub id: i64,

    /// Conversation the message belongs to.
    pub chat: Chat,

    #[serde(default)]
    pub from: Option<User>,

    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BotCommand {
    /// Text of the command; 1-32 characters. Can contain only lowercase English letters, digits and underscores.
    pub command: Cow<'static, str>,

    /// Description of the command; 1-256 characters.
    pub description: Cow<'static, str>,
}

/// https://core.telegram.org/bots/api#formatting-options
#[derive(Debug, Serialize)]
pub enum ParseMode {
    /// https://core.telegram.org/bots/api#markdownv2-style
    MarkdownV2,

    /// https://core.telegram.org/bots/api#html-style
    #[serde(rename = "HTML")]
    Html,
}
