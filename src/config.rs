use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use egg_mode::KeyPair;
use eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

const ARCHIVE_PATH: &str = "test_responses2";
const SETTINGS_FILE: &str = "twitter_settings.json";
const PAGING_FILE: &str = "paging_positions.json";

type PagingPositions = HashMap<String, u64>;

#[derive(Clone, Debug)]
pub struct Config {
    /// If this is enabled, it will only check for new data and not continue
    /// paging for old data. (e.g. only activate this once a full archive)
    /// has been established
    pub is_sync: bool,
    pub token: egg_mode::Token,
    config_data: ConfigData,
    /// Remember the paging positions for the different endpoints,
    /// so that restarting the crawler will continue where it left off.
    paging_positions: Arc<Mutex<PagingPositions>>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.is_sync == other.is_sync && self.config_data == other.config_data
    }
}

impl Config {
    pub fn archive_path() -> PathBuf {
        PathBuf::from_str(ARCHIVE_PATH).unwrap()
    }

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
    fn keypair() -> KeyPair {
        let consumer_key = obfstr::obfstr!(include_str!("../API_KEY"))
            .trim()
            .to_string();
        let consumer_secret = obfstr::obfstr!(include_str!("../API_SECRET"))
            .trim()
            .to_string();

        egg_mode::KeyPair::new(consumer_key, consumer_secret)
    }
    pub fn open() -> Result<Self> {
        let con_token = Self::keypair();

        let (token, config_data, paging_positions) = {
            let fp = std::fs::File::open(SETTINGS_FILE)?;
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

            (token, config_data, paging_positions)
        };

        Ok(Config {
            token,
            config_data,
            paging_positions: Arc::new(Mutex::new(paging_positions)),
            is_sync: false,
        })
    }

    pub async fn verify(&self) -> Result<()> {
        Ok(egg_mode::auth::verify_tokens(&self.token)
            .await
            .map(|_| ())?)
    }

    pub async fn load() -> Result<Self> {
        let a1 = Config::load_inner().await;
        if let Ok(conf) = a1 {
            return Ok(conf);
        }

        Config::load_inner().await
    }

    async fn load_inner() -> Result<Self> {
        if let Ok(config) = Self::open() {
            if let Err(err) = config.verify().await {
                println!("We've hit an error using your old tokens: {:?}", err);
                println!("We'll have to reauthenticate before continuing.");
                println!("Last Paging Positions");
                std::fs::remove_file(SETTINGS_FILE).unwrap();
                bail!("Please Relogin")
            } else {
                println!("Logged in as {}", config.config_data.username);
            }
            Ok(config)
        } else {
            println!("Request Token");
            let request_data = RequestData::request().await?;

            println!("Go to the following URL, sign in, and give me the PIN that comes back:");
            println!("{}", request_data.authorize_url);

            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            println!();

            let config = request_data.validate(&pin).await?;
            config.config_data.write()?;
            Ok(config)
        }
    }
}

#[derive(Clone)]
pub struct RequestData {
    request_token: KeyPair,
    pub authorize_url: String,
    user_pin: String,
}

impl RequestData {
    pub async fn request() -> Result<Self> {
        let con_token = Config::keypair();
        let request_token = egg_mode::auth::request_token(&con_token, "oob").await?;
        let authorize_url = egg_mode::auth::authorize_url(&request_token);

        Ok(Self {
            request_token,
            authorize_url,
            user_pin: String::new(),
        })
    }

    pub async fn validate(&self, pin: &str) -> Result<Config> {
        let con_token = Config::keypair();
        let (token, user_id, username) =
            egg_mode::auth::access_token(con_token, &self.request_token, pin).await?;

        let config_data = match token {
            egg_mode::Token::Access {
                access: ref access_token,
                ..
            } => ConfigData {
                username,
                user_id,
                key: access_token.key.to_string(),
                secret: access_token.secret.to_string(),
                crawl_options: Default::default(),
            },
            _ => bail!("Invalid Token Type {token:?}"),
        };

        config_data.write()?;

        Ok(Config {
            token,
            config_data,
            paging_positions: Default::default(),
            is_sync: false,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ConfigData {
    username: String,
    user_id: u64,
    key: String,
    secret: String,
    #[serde(default)]
    crawl_options: CrawlOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
            tweet_responses: false,
            tweet_profiles: false,
            mentions: false,
            followers: false,
            follows: false,
            lists: false,
            media: false,
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
