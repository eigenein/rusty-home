use std::io::ErrorKind;

use anyhow::{Context, Error, Result};
use futures::{AsyncBufReadExt, Stream, TryStreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client};
use serde_json::json;
use tracing::{error, info, instrument};

use crate::models::{Message, Token};

const USER_AGENT: &str = concat!(
    "rusty-tractive/",
    env!("CARGO_PKG_VERSION"),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

#[must_use]
pub struct Api {
    client: Client,
}

impl Api {
    #[instrument(skip_all)]
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json;charset=UTF-8"),
        );
        headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("application/json"));
        headers.insert("X-Tractive-Client", HeaderValue::from_static("625e533dc3c3b41c28a669f0"));
        let client = Client::builder()
            .gzip(true)
            .default_headers(headers)
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self { client })
    }

    #[instrument(skip_all, fields(email = email))]
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<Token> {
        let token: Token = self
            .client
            .post("https://graph.tractive.com/3/auth/token")
            .json(&json! ({
                "platform_email": email,
                "platform_token": password,
                "grant_type": "tractive",
            }))
            .send()
            .await
            .context("failed to send the authentication request")?
            .error_for_status()
            .context("authentication failed")?
            .json()
            .await
            .context("failed to deserialize the authentication token")?;
        info!(expires_at = ?token.expires_at, "authenticated");
        Ok(token)
    }

    #[instrument(skip_all, fields(user_id = user_id))]
    pub async fn get_messages(
        &self,
        user_id: &str,
        access_token: &str,
    ) -> Result<impl Stream<Item = Result<Message>>> {
        let stream = self
            .client
            .post("https://channel.tractive.com/3/channel")
            .header("X-Tractive-User", user_id)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("failed to send the channel request")?
            .error_for_status()
            .context("the channel request failed")?
            .bytes_stream()
            .map_err(|error| std::io::Error::new(ErrorKind::Other, error))
            .into_async_read()
            .lines()
            .try_filter_map(|line| async move {
                match serde_json::from_str(&line).context("failed to deserialize") {
                    Ok(message) => Ok(Some(message)),
                    Err(error) => {
                        error!("{:#}: {}", error, line);
                        Ok(None)
                    }
                }
            })
            .map_err(Error::from);
        Ok(stream)
    }
}
