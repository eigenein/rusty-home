use anyhow::Result;
use clap::Parser;
use reqwest::{Client, Url};
use tracing::{debug, instrument, warn};

#[derive(Parser)]
pub struct Opts {
    /// URL to which the microservice should post its heartbeat
    #[clap(long = "heartbeat-url", env = "RUSTY_HOME_HEARTBEAT_URL")]
    url: Option<String>,
}

impl Opts {
    pub fn get_heartbeat(self) -> Result<Heartbeat> {
        if self.url.is_none() {
            warn!("heartbeat URL is not specified, heartbeat is disabled");
        }
        self.url
            .map(|url| Ok(Heartbeat(Some((Client::new(), url.parse()?)))))
            .unwrap_or_else(|| Ok(Heartbeat(None)))
    }
}

pub struct Heartbeat(Option<(Client, Url)>);

impl Heartbeat {
    #[instrument(level = "debug", skip_all)]
    pub async fn send(&self) {
        let (client, url) = match &self.0 {
            Some(client) => client,
            None => {
                debug!("ignoring the heartbeat");
                return;
            }
        };
        if let Err(error) = client.post(url.clone()).send().await {
            warn!("heartbeat error: {:#}", error);
        }
    }
}
