use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use std::time::Instant;
use tracing::{debug, instrument};

use crate::models;

const USER_AGENT: &str = concat!(
    "rusty-shared-telegram/",
    env!("VERGEN_GIT_SHA_SHORT"),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

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

    #[instrument(level = "debug", skip_all, err)]
    pub async fn get_updates(
        &self,
        offset: i64,
        timeout: std::time::Duration,
    ) -> Result<Vec<models::Update>> {
        let start_time = Instant::now();
        let response = self
            .client
            .post(format!(
                "https://api.telegram.org/bot{}/getUpdates",
                self.token,
            ))
            .json(&json!({
                "offset": offset,
                "timeout": timeout.as_secs(),
            }))
            .timeout(self.timeout + timeout)
            .send()
            .await
            .context("failed to send the request")?
            .json::<models::Response<Vec<models::Update>>>()
            .await
            .context("failed to deserialize response")?;
        match response {
            models::Response::Err {
                error_code: 409, ..
            } => {
                let time_left = timeout - start_time.elapsed();
                debug!(secs = time_left.as_secs(), "conflict, sleeping…");
                async_std::task::sleep(time_left).await;
                Ok(Vec::new())
            }
            _ => response.into(),
        }
    }

    #[instrument(level = "debug", skip_all, fields(method_name = method_name))]
    async fn call<R: DeserializeOwned>(
        &self,
        method_name: &str,
        body: &impl Serialize,
    ) -> Result<R> {
        self.client
            .post(format!(
                "https://api.telegram.org/bot{}/{}",
                self.token, method_name,
            ))
            .json(body)
            .send()
            .await
            .with_context(|| format!("failed to send the `{}` request", method_name))?
            .error_for_status()
            .with_context(|| format!("`{}` request failed", method_name))?
            .json::<models::Response<R>>()
            .await
            .with_context(|| format!("failed to deserialize `{}` response", method_name))?
            .into()
    }
}
