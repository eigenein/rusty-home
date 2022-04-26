use anyhow::{anyhow, Result};
use serde::Deserialize;

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

    #[serde(default)]
    pub from: Option<User>,

    #[serde(default)]
    pub text: Option<String>,
}
