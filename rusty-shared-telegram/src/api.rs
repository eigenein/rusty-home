use std::time;

use anyhow::Result;
use reqwest::Client;
use tracing::instrument;

const USER_AGENT: &str = concat!(
    "rusty-shared-telegram/",
    env!("CARGO_PKG_VERSION"),
    " (Rust; https://github.com/eigenein/rusty-home)"
);

#[derive(Clone)]
pub struct BotApi {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) timeout: time::Duration,
}

impl BotApi {
    #[instrument(level = "debug", skip_all)]
    pub fn new(token: String, timeout: time::Duration) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(timeout)
            .build()?;
        let this = Self {
            client,
            base_url: format!("https://api.telegram.org/bot{}", token),
            timeout,
        };
        Ok(this)
    }
}
