use anyhow::Result;
use async_std::sync::Mutex;
use clap::Parser;
use reqwest::{Client, Url};
use std::time::Instant;
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
            .map(|url| Ok(Heartbeat::new(Some((Client::new(), url.parse()?)))))
            .unwrap_or_else(|| Ok(Heartbeat::new(None)))
    }
}

pub struct Heartbeat {
    endpoint: Option<(Client, Url)>,
    last_beat_timestamp: Mutex<Instant>,
}

impl Heartbeat {
    const INTERVAL: ::std::time::Duration = ::std::time::Duration::from_secs(60);

    pub fn new(endpoint: Option<(Client, Url)>) -> Self {
        Self {
            endpoint,
            last_beat_timestamp: Mutex::new(Instant::now() - Self::INTERVAL),
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn send(&self) {
        let mut last_beat_timestamp = self.last_beat_timestamp.lock().await;
        if last_beat_timestamp.elapsed() < Self::INTERVAL {
            debug!(
                elapsed = format!("{:?}", last_beat_timestamp.elapsed()).as_str(),
                "heartbeat is rate-limited",
            );
            return;
        }

        match &self.endpoint {
            Some((client, url)) => {
                if let Err(error) = client.post(url.clone()).send().await {
                    warn!("heartbeat error: {:#}", error);
                }
            }
            None => {
                debug!("heartbeat is disabled");
            }
        };
        *last_beat_timestamp = Instant::now();
    }
}
