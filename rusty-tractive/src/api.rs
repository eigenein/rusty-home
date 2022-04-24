use crate::models::Token;
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client};
use serde_json::json;

const USER_AGENT: &str = concat!(
    "rusty-tractive/",
    clap::crate_version!(),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

#[must_use]
pub struct Api {
    email: String,
    password: String,
    client: Client,
}

impl Api {
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json;charset=UTF-8"),
        );
        headers.insert(
            header::ACCEPT_ENCODING,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-Tractive-Client",
            HeaderValue::from_static("625e533dc3c3b41c28a669f0"),
        );
        let client = Client::builder()
            .gzip(true)
            .default_headers(headers)
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            email: email.into(),
            password: password.into(),
            client,
        })
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub async fn authenticate(&mut self) -> Result<()> {
        tracing::info!("authenticating…");
        let token: Token = self
            .client
            .post("https://graph.tractive.com/3/auth/token")
            .json(&json! ({
                "platform_email": self.email,
                "platform_token": self.password,
                "grant_type": "tractive",
            }))
            .send()
            .await
            .context("authentication request failed")?
            .json()
            .await
            .context("failed to deserialize the authentication token")?;
        tracing::info!(
            expires_at = token.expires_at.to_string().as_str(),
            "authenticated",
        );
        Ok(())
    }
}
