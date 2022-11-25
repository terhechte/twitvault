use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

const SETTINGS_FILE: &str = "twitter_settings.json";
const PAGING_FILE: &str = "paging_positions.json";

type PagingPositions = HashMap<String, u64>;

#[derive(Clone)]
pub struct Config {
    pub token: egg_mode::Token,
    config_data: ConfigData,
    /// Remember the paging positions for the different endpoints,
    /// so that restarting the crawler will continue where it left off.
    paging_positions: Arc<Mutex<PagingPositions>>,
}

impl Config {
    pub fn screen_name(&self) -> &str {
        &self.config_data.username
    }

    pub fn user_id(&self) -> u64 {
        self.config_data.user_id
    }

    pub fn crawl_options(&self) -> &CrawlOptions {
        &self.config_data.crawl_options
    }
}

impl Config {
    pub fn paging_position(&self, key: &str) -> Option<u64> {
        self.paging_positions.lock().ok()?.get(key).copied()
    }

    pub fn set_paging_position(&self, key: &str, value: Option<u64>) {
        let Ok(mut lock) = self.paging_positions.lock() else { return };
        if let Some(value) = value {
            lock.insert(key.to_string(), value);
        } else {
            lock.remove(key);
        }
        let Ok(f) = std::fs::File::create(PAGING_FILE) else {
            warn!("Could not create / save {PAGING_FILE}");
            return
        };
        if let Err(e) = serde_json::to_writer(f, &(*lock)) {
            warn!("Could not serialize {PAGING_FILE}: {e:?}");
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let a1 = Config::load_inner().await;
        if let Ok(conf) = a1 {
            return Ok(conf);
        }

        Config::load_inner().await
    }

    async fn load_inner() -> Result<Self> {
        let consumer_key = obfstr::obfstr!(include_str!("../API_KEY"))
            .trim()
            .to_string();
        let consumer_secret = obfstr::obfstr!(include_str!("../API_SECRET"))
            .trim()
            .to_string();

        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let (token, config_data, paging_positions) = if let Ok(fp) =
            std::fs::File::open(SETTINGS_FILE)
        {
            let config_data: ConfigData = serde_json::from_reader(fp)?;
            let paging_positions: PagingPositions = std::fs::File::open(PAGING_FILE)
                .map_err(|e| eyre::eyre!("{e:?}"))
                .and_then(|e| serde_json::from_reader(e).map_err(|e| eyre::eyre!("{e:?}")))
                .unwrap_or_default();

            let access_token =
                egg_mode::KeyPair::new(config_data.key.clone(), config_data.secret.clone());
            let token = egg_mode::Token::Access {
                consumer: con_token,
                access: access_token,
            };

            if let Err(err) = egg_mode::auth::verify_tokens(&token).await {
                println!("We've hit an error using your old tokens: {:?}", err);
                println!("We'll have to reauthenticate before continuing.");
                println!("Last Paging Positions");
                std::fs::remove_file(SETTINGS_FILE).unwrap();
            } else {
                println!("Logged in as {}", config_data.username);
            }
            (token, config_data, paging_positions)
        } else {
            println!("Request Token");
            let request_token = egg_mode::auth::request_token(&con_token, "oob").await?;

            println!("Go to the following URL, sign in, and give me the PIN that comes back:");
            println!("{}", egg_mode::auth::authorize_url(&request_token));

            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            println!();

            let tok_result = egg_mode::auth::access_token(con_token, &request_token, pin).await?;

            let token = tok_result.0;

            let config_data = match token {
                egg_mode::Token::Access {
                    access: ref access_token,
                    ..
                } => {
                    let config_data = ConfigData {
                        username: tok_result.2,
                        user_id: tok_result.1,
                        key: access_token.key.to_string(),
                        secret: access_token.secret.to_string(),
                        crawl_options: Default::default(),
                    };
                    config_data.write()?;
                    println!("Saved settings to {}", SETTINGS_FILE);
                    config_data
                }
                _ => bail!("Invalid State"),
            };
            (token, config_data, PagingPositions::default())
        };

        if std::path::Path::new(SETTINGS_FILE).exists() {
            Ok(Config {
                token,
                config_data,
                paging_positions: Arc::new(Mutex::new(paging_positions)),
            })
        } else {
            bail!("Could not log in")
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ConfigData {
    username: String,
    user_id: u64,
    key: String,
    secret: String,
    #[serde(default)]
    crawl_options: CrawlOptions,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CrawlOptions {
    /// Download all tweets
    pub tweets: bool,
    /// Download the first 50 responses to a tweet
    pub tweet_responses: bool,
    /// Download the profile of the tweet author
    pub tweet_profiles: bool,
    /// Download all mentions
    pub mentions: bool,
    /// Download all followers + profiles
    pub followers: bool,
    /// Download all follows + profiles
    pub follows: bool,
    /// Download lists as well as the profiles of the members
    pub lists: bool,
    /// Download media from tweets and profiles
    pub media: bool,
}

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            tweets: true,
            tweet_responses: true,
            tweet_profiles: false,
            mentions: true,
            followers: true,
            follows: true,
            lists: true,
            media: true,
        }
    }
}

impl ConfigData {
    fn write(&self) -> Result<()> {
        let f = std::fs::File::create(SETTINGS_FILE)?;
        serde_json::to_writer(f, &self)?;
        Ok(())
    }
}
