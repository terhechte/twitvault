use dotenv_codegen::dotenv;
use eyre::{bail, Result};
use serde::{Deserialize, Serialize};

const SETTINGS_FILE: &str = "twitter_settings.json";

#[derive(Clone)]
pub struct Config {
    pub token: egg_mode::Token,
    pub user_id: u64,
    pub screen_name: String,
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
        let consumer_key = dotenv!("KEY");
        let consumer_secret = dotenv!("SECRET");

        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let user_id: u64;
        let username: String;
        let token: egg_mode::Token;

        if let Ok(fp) = std::fs::File::open(SETTINGS_FILE) {
            let config_data: ConfigData = serde_json::from_reader(fp)?;

            username = config_data.username.clone();
            user_id = config_data.user_id;
            let access_token =
                egg_mode::KeyPair::new(config_data.key.clone(), config_data.secret.clone());
            token = egg_mode::Token::Access {
                consumer: con_token,
                access: access_token,
            };

            if let Err(err) = egg_mode::auth::verify_tokens(&token).await {
                println!("We've hit an error using your old tokens: {:?}", err);
                println!("We'll have to reauthenticate before continuing.");
                std::fs::remove_file(SETTINGS_FILE).unwrap();
            } else {
                println!("Logged in as {}", username);
            }
        } else {
            let request_token = egg_mode::auth::request_token(&con_token, "oob").await?;

            println!("Go to the following URL, sign in, and give me the PIN that comes back:");
            println!("{}", egg_mode::auth::authorize_url(&request_token));

            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            println!();

            let tok_result = egg_mode::auth::access_token(con_token, &request_token, pin)
                .await
                .unwrap();

            token = tok_result.0;
            user_id = tok_result.1;
            username = tok_result.2;

            match token {
                egg_mode::Token::Access {
                    access: ref access_token,
                    ..
                } => {
                    let config_data = ConfigData {
                        username: username.clone(),
                        user_id,
                        key: access_token.key.to_string(),
                        secret: access_token.secret.to_string(),
                    };
                    let f = std::fs::File::create(SETTINGS_FILE)?;
                    serde_json::to_writer(f, &config_data)?;
                    println!("Saved settings to {}", SETTINGS_FILE);
                }
                _ => bail!("Invalid State"),
            }
        }

        if std::path::Path::new(SETTINGS_FILE).exists() {
            Ok(Config {
                token,
                user_id,
                screen_name: username,
            })
        } else {
            bail!("Could not log in")
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ConfigData {
    username: String,
    user_id: u64,
    key: String,
    secret: String,
}
