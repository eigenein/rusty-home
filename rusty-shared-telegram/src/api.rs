use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use tracing::instrument;

use crate::models;

const USER_AGENT: &str = concat!(
    "rusty-shared-telegram/",
    env!("VERGEN_GIT_SHA_SHORT"),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

pub struct Api {
    client: Client,
    token: String,
}

impl Api {
    #[instrument(level = "debug", skip_all)]
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        Ok(Self { client, token })
    }

    /// https://core.telegram.org/bots/api#getme
    #[instrument(level = "info", skip_all)]
    pub async fn get_me(&self) -> Result<models::User> {
        self.call("getMe", &()).await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_updates(&self, timeout: ::std::time::Duration) -> Result<Vec<models::Update>> {
        self.call(
            "getUpdates",
            &json!({
                "timeout": timeout.as_secs(),
            }),
        )
        .await
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
            .context("failed to send the request")?
            .error_for_status()
            .context("HTTP request failed")?
            .json::<models::Response<R>>()
            .await
            .context("failed to deserialize response")?
            .into()
    }
}
