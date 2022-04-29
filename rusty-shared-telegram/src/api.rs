use std::time::Instant;

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

    #[instrument(level = "debug", skip_all, err)]
    pub async fn get_updates(&self, payload: methods::GetUpdates) -> Result<Vec<models::Update>> {
        let start_time = Instant::now();
        let response = self
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
            .json::<models::Response<Vec<models::Update>>>()
            .await
            .context("failed to deserialize response")?;
        match response {
            models::Response::Err {
                error_code: 409, ..
            } => {
                let time_left = payload.timeout - start_time.elapsed();
                debug!(secs = time_left.as_secs(), "conflict, sleeping…");
                async_std::task::sleep(time_left).await;
                Ok(Vec::new())
            }
            _ => response.into(),
        }
    }

    /// https://core.telegram.org/bots/api#setmycommands
    #[instrument(level = "info", skip_all)]
    pub async fn set_my_commands(&self, payload: methods::SetMyCommands) -> Result<bool> {
        self.call("setMyCommands", &payload).await
    }

    /// https://core.telegram.org/bots/api#sendmessage
    #[instrument(level = "info", skip_all)]
    pub async fn send_message(&self, payload: methods::SendMessage) -> Result<models::Message> {
        self.call("sendMessage", &payload).await
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
            .json::<models::Response<R>>()
            .await
            .with_context(|| format!("failed to deserialize `{}` response", method_name))?
            .into()
    }
}
