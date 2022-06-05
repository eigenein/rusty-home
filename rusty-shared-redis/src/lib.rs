use std::fmt::Debug;
use std::time;

use anyhow::{Context, Result};
use async_std::future::timeout;
use fred::prelude::*;
use fred::types::{MultipleKeys, MultipleValues, PerformanceConfig, RedisKey};
use tracing::{debug, instrument};

pub struct Redis {
    pub client: RedisClient,
    script_hashes: ScriptHashes,
}

#[derive(Debug, Clone)]
struct ScriptHashes {
    set_if_greater: String,
    set_if_not_equal: String,
    create_consumer_group: String,
}

impl Redis {
    /// `EVALSHA` seems to get stuck on a master failure.
    /// Thus, I put a short timeout on each `EVALSHA` call.
    const EVALSHA_TIMEOUT: time::Duration = time::Duration::from_secs(5);

    #[instrument(skip_all, fields(url = url))]
    pub async fn connect(url: &str) -> Result<Self> {
        let config = {
            let mut config = RedisConfig::from_url(url)?;
            config.blocking = Blocking::Error;
            config.tracing = true;
            config.performance = PerformanceConfig {
                pipeline: false,
                ..Default::default()
            };
            config
        };

        let client = RedisClient::new(config);
        connect(&client).await?;
        // TODO: client.client_setname("fred").await?; // FIXME: use bin crate name.
        let script_hashes = load_scripts(&client).await?;

        Ok(Self {
            client,
            script_hashes,
        })
    }

    #[instrument(skip_all)]
    pub async fn clone(&self) -> Result<Self> {
        let client = self.client.clone_new();
        connect(&client).await?;
        let this = Self {
            client,
            script_hashes: self.script_hashes.clone(),
        };
        Ok(this)
    }

    #[instrument(skip_all, fields(key = ?key, group_name = group_name))]
    pub async fn create_consumer_group<K: Into<RedisKey> + Debug>(
        &self,
        key: K,
        group_name: &str,
    ) -> Result<bool> {
        timeout(
            Self::EVALSHA_TIMEOUT,
            self.client
                .evalsha(&self.script_hashes.create_consumer_group, key, group_name),
        )
        .await
        .context("timeout while creating the consumer group")?
        .context("failed to create the consumer group")
    }

    #[instrument(skip_all, fields(key = ?key))]
    pub async fn set_if_greater<K, V>(&self, key: K, value: V) -> Result<(bool, Option<V>)>
    where
        K: Debug + Into<MultipleKeys>,
        V: FromRedis + Unpin + Send,
        V: 'static + TryInto<MultipleValues>,
        V::Error: Into<RedisError>,
    {
        timeout(
            Self::EVALSHA_TIMEOUT,
            self.client
                .evalsha(&self.script_hashes.set_if_greater, key, value),
        )
        .await
        .context("timed out while calling set-if-greater")?
        .context("failed to set-if-greater")
    }

    #[instrument(skip_all, fields(key = ?key))]
    pub async fn set_if_not_equal<K, V>(&self, key: K, value: V) -> Result<(bool, Option<V>)>
    where
        K: Debug + Into<MultipleKeys>,
        V: FromRedis + Unpin + Send,
        V: 'static + TryInto<MultipleValues>,
        V::Error: Into<RedisError>,
    {
        timeout(
            Self::EVALSHA_TIMEOUT,
            self.client
                .evalsha(&self.script_hashes.set_if_not_equal, key, value),
        )
        .await
        .context("timed out while calling set-if-not-equal")?
        .context("failed to set-if-not-equal")
    }
}

#[instrument(skip_all)]
async fn connect(client: &RedisClient) -> Result<()> {
    client.connect(None);
    debug!("awaiting connection…");
    client
        .wait_for_connect()
        .await
        .context("failed to connect to Redis")?;
    debug!("connected to Redis");
    Ok(())
}

#[instrument(skip_all)]
async fn load_scripts(client: &RedisClient) -> Result<ScriptHashes> {
    let set_if_greater = client.script_load(SET_IF_GREATER_SCRIPT).await?;
    let create_consumer_group = client.script_load(CREATE_CONSUMER_GROUP).await?;
    let set_if_not_equal = client.script_load(SET_IF_NOT_EQUAL_SCRIPT).await?;

    let hashes = ScriptHashes {
        set_if_greater,
        create_consumer_group,
        set_if_not_equal,
    };

    debug!(hashes = ?hashes, "loaded the scripts");
    Ok(hashes)
}

/// Set value, if it's greater than the stored one if any.
// language=lua
const SET_IF_GREATER_SCRIPT: &str = r#"
    local new_value = tonumber(ARGV[1]);
    local last_value = redis.call("GET", KEYS[1]);

    if last_value == false or tonumber(last_value) < new_value then
        redis.call("SET", KEYS[1], new_value);
        return {1, last_value}
    else
        return {0, last_value}
    end
"#;

// language=lua
const SET_IF_NOT_EQUAL_SCRIPT: &str = r#"
    local new_value = ARGV[1];
    local last_value = redis.call("GET", KEYS[1]);

    if last_value ~= new_value then
        redis.call("SET", KEYS[1], new_value);
        return {1, last_value}
    else
        return {0, last_value}
    end
"#;

/// Create a consumer group, if not exists.
// language=lua
const CREATE_CONSUMER_GROUP: &str = r#"
    for _, group_info in ipairs(redis.call("XINFO", "GROUPS", KEYS[1])) do
        for i = 1, #group_info, 2 do
            if group_info[i] == "name" and group_info[i + 1] == ARGV[1] then
                -- The group already exists.
                return 0
            end
        end
    end

    redis.call("XGROUP", "CREATE", KEYS[1], ARGV[1], "$", "MKSTREAM")
    return 1
"#;
