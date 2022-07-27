use anyhow::Result;
use clap::Parser;
use reqwest::{Client, Url};
use tracing::{debug, instrument, warn};

#[derive(Parser)]
pub struct Opts {
    /// URL to which the microservice should post its heartbeat.
    #[clap(long = "heartbeat-url", env = "RUSTY_HOME_HEARTBEAT_URL")]
    heartbeat_url: Option<String>,
}

impl Opts {
    pub fn get_heartbeat(self) -> Result<Heartbeat> {
        if self.heartbeat_url.is_none() {
            warn!("heartbeat URL is not specified, heartbeat is disabled");
        }
        self.heartbeat_url
            .map(|url| Ok(Heartbeat::new(Some((Client::new(), url.parse()?)))))
            .unwrap_or_else(|| Ok(Heartbeat::new(None)))
    }
}

/// TODO: extract to a separate package. Use `governor`.
pub struct Heartbeat {
    endpoint: Option<(Client, Url)>,
}

impl Heartbeat {
    pub fn new(endpoint: Option<(Client, Url)>) -> Self {
        Self { endpoint }
    }

    #[instrument(skip_all)]
    pub async fn send(&self) {
        match &self.endpoint {
            Some((client, url)) => {
                debug!("sending heartbeat…");
                if let Err(error) = client.post(url.clone()).send().await {
                    warn!("heartbeat error: {:#}", error);
                }
            }
            None => {
                debug!("heartbeat is disabled");
            }
        };
    }
}
