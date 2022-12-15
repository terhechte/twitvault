#![allow(unused)]

use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use dotenvy_macro::dotenv;
use egg_mode::KeyPair;
use eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

const ARCHIVE_PATH: &str = "archive";
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
    pub config_data: ConfigData,
    /// Remember the paging positions for the different endpoints,
    /// so that restarting the crawler will continue where it left off.
    paging_positions: Arc<Mutex<PagingPositions>>,
    /// If this is a config for a custom path
    custom_path: Option<PathBuf>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.is_sync == other.is_sync && self.config_data == other.config_data
    }
}

impl Eq for Config {}

impl Config {
    /// The storage path for a given initialized config.
    /// will include a `custom path` if that has been
    /// set by the user at runtime
    pub fn actual_storage_path(&self) -> PathBuf {
        Self::storage_path(self.custom_path.clone())
    }

    /// The default storage path *or* the custom path which
    /// a user can set at runtime
    pub fn storage_path(custom: Option<PathBuf>) -> PathBuf {
        custom.unwrap_or_else(data_directory).join(ARCHIVE_PATH)
    }

    /// The path to the config file which is within
    /// the `storage_path`
    pub fn config_path(custom: Option<PathBuf>) -> PathBuf {
        Config::storage_path(custom).join(SETTINGS_FILE)
    }

    /// The path to the paging file
    pub fn paging_path(custom: Option<PathBuf>) -> PathBuf {
        Config::storage_path(custom).join(PAGING_FILE)
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

    pub fn set_crawl_options(&mut self, options: &CrawlOptions) {
        self.config_data.crawl_options = options.clone();
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
        let paging_path = Config::paging_path(self.custom_path.clone());
        let Ok(f) = std::fs::File::create(paging_path.clone()) else {
            warn!("Could not create / save {}", &paging_path.display());
            return
        };
        if let Err(e) = serde_json::to_writer(f, &(*lock)) {
            warn!("Could not serialize {}: {e:?}", &paging_path.display());
        }
    }
}

impl Config {
    fn keypair() -> KeyPair {
        // somehow dotenv and dotenvy doesn't behave as expected. need to look into it:
        // https://github.com/dotenv-rs/dotenv/issues/71
        let consumer_key = obfstr::obfstr!(env!("API_KEY")).trim().to_string();
        let consumer_secret = obfstr::obfstr!(env!("API_SECRET")).trim().to_string();

        egg_mode::KeyPair::new(consumer_key, consumer_secret)
    }
    pub fn open(custom_path: Option<PathBuf>) -> Result<Self> {
        let con_token = Self::keypair();

        let (token, config_data, paging_positions) = {
            // if we can't find the path in the archive (default),
            // then try in the parent directory (backwards compatibility)
            let mut path = Config::config_path(custom_path.clone());
            if !path.exists() {
                let old_path = Config::storage_path(custom_path.clone())
                    .parent()
                    .map(|e| e.to_owned())
                    .ok_or(eyre::eyre!("No root folder for storage path"))?;
                let old_config_path = old_path.join(SETTINGS_FILE);
                if old_config_path.exists() {
                    path = old_config_path;
                } else {
                    bail!(
                        "Could not find config file in either {} or {}",
                        path.display(),
                        old_config_path.display()
                    )
                }
            }
            let fp = std::fs::File::open(path)?;
            let config_data: ConfigData = serde_json::from_reader(fp)?;
            let paging_path = Self::paging_path(custom_path.clone());
            let paging_positions: PagingPositions = std::fs::File::open(paging_path)
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
            custom_path,
        })
    }

    pub async fn verify(&self) -> Result<()> {
        Ok(egg_mode::auth::verify_tokens(&self.token)
            .await
            .map(|_| ())?)
    }

    pub async fn load(custom_path: Option<PathBuf>) -> Result<Self> {
        let a1 = Config::load_inner(custom_path.clone()).await;
        if let Ok(conf) = a1 {
            return Ok(conf);
        }

        Config::load_inner(custom_path).await
    }

    async fn load_inner(custom_path: Option<PathBuf>) -> Result<Self> {
        if let Ok(config) = Self::open(custom_path.clone()) {
            if let Err(err) = config.verify().await {
                println!("We've hit an error using your old tokens: {:?}", err);
                println!("We'll have to reauthenticate before continuing.");
                println!("Last Paging Positions");
                let path = Config::config_path(custom_path.clone());
                std::fs::remove_file(path).unwrap();
                bail!("Please Relogin")
            } else {
                println!("Logged in as {}", config.config_data.username);
            }
            Ok(config)
        } else {
            println!("Request Token");
            let request_data = RequestData::request(custom_path.clone()).await?;

            println!("Go to the following URL, sign in, and give me the PIN that comes back:");
            println!("{}", request_data.authorize_url);

            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            println!();

            let config = request_data.validate(&pin).await?;
            config.config_data.write(custom_path.clone())?;
            Ok(config)
        }
    }
}

#[derive(Clone)]
pub struct RequestData {
    request_token: KeyPair,
    pub authorize_url: String,
    user_pin: String,
    custom_path: Option<PathBuf>,
}

impl RequestData {
    pub async fn request(custom_path: Option<PathBuf>) -> Result<Self> {
        let con_token = Config::keypair();
        let request_token = egg_mode::auth::request_token(&con_token, "oob").await?;
        let authorize_url = egg_mode::auth::authorize_url(&request_token);

        Ok(Self {
            request_token,
            authorize_url,
            user_pin: String::new(),
            custom_path,
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

        config_data.write(self.custom_path.clone())?;

        Ok(Config {
            token,
            config_data,
            paging_positions: Default::default(),
            is_sync: false,
            custom_path: self.custom_path.clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfigData {
    pub username: String,
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
    /// Download the liked tweets and profiles for a user
    #[serde(default)]
    pub likes: bool,
}

impl CrawlOptions {
    pub fn changed(&self, change: impl FnOnce(&mut Self)) -> Self {
        let mut copy = self.clone();
        change(&mut copy);
        copy
    }
}

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            tweets: true,
            tweet_responses: false,
            tweet_profiles: true,
            mentions: true,
            followers: true,
            follows: true,
            lists: false,
            media: true,
            likes: true,
        }
    }
}

impl ConfigData {
    fn write(&self, custom_path: Option<PathBuf>) -> Result<()> {
        let path = Config::config_path(custom_path);
        let f = std::fs::File::create(&path)?;
        serde_json::to_writer(f, &self)?;
        Ok(())
    }
}

fn data_directory() -> PathBuf {
    use directories_next::ProjectDirs;
    if let Some(proj_dirs) = ProjectDirs::from("com", "StyleMac", "TwitVault") {
        proj_dirs.config_dir().to_path_buf()
    } else {
        panic!("Couldn't find a folder to save the data")
    }
}
