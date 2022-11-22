use dotenv_codegen::dotenv;
use std::io::{Read, Write};

//This is not an example that can be built with cargo! This is some helper code for the other
//examples so they can load access keys from the same place.

pub struct Config {
    pub token: egg_mode::Token,
    pub user_id: u64,
    pub screen_name: String,
}

impl Config {
    pub async fn load() -> Self {
        let a1 = Config::load_inner().await;
        if let Some(conf) = a1 {
            return conf;
        }

        Config::load_inner().await.unwrap()
    }

    /// This needs to be a separate function so we can retry after creating the
    /// twitter_settings file. Idealy we would recurse, but that requires boxing
    /// the output which doesn't seem worthwhile

    async fn load_inner() -> Option<Self> {
        //IMPORTANT: make an app for yourself at apps.twitter.com and get your
        //key/secret into these files; these examples won't work without them
        let consumer_key = dotenv!("KEY");
        let consumer_secret = dotenv!("SECRET");

        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let mut config = String::new();
        let user_id: u64;
        let username: String;
        let token: egg_mode::Token;

        //look at all this unwrapping! who told you it was my birthday?
        if let Ok(mut f) = std::fs::File::open("twitter_settings") {
            f.read_to_string(&mut config).unwrap();

            let mut iter = config.split('\n');

            username = iter.next().unwrap().to_string();
            user_id = u64::from_str_radix(&iter.next().unwrap(), 10).unwrap();
            let access_token = egg_mode::KeyPair::new(
                iter.next().unwrap().to_string(),
                iter.next().unwrap().to_string(),
            );
            token = egg_mode::Token::Access {
                consumer: con_token,
                access: access_token,
            };

            if let Err(err) = egg_mode::auth::verify_tokens(&token).await {
                println!("We've hit an error using your old tokens: {:?}", err);
                println!("We'll have to reauthenticate before continuing.");
                std::fs::remove_file("twitter_settings").unwrap();
            } else {
                println!("Welcome back, {}!\n", username);
            }
        } else {
            let request_token = egg_mode::auth::request_token(&con_token, "oob")
                .await
                .unwrap();

            println!("Go to the following URL, sign in, and give me the PIN that comes back:");
            println!("{}", egg_mode::auth::authorize_url(&request_token));

            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            println!("");

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
                    config.push_str(&username);
                    config.push('\n');
                    config.push_str(&format!("{}", user_id));
                    config.push('\n');
                    config.push_str(&access_token.key);
                    config.push('\n');
                    config.push_str(&access_token.secret);
                }
                _ => unreachable!(),
            }

            let mut f = std::fs::File::create("twitter_settings").unwrap();
            f.write_all(config.as_bytes()).unwrap();

            println!("Welcome, {}, let's get this show on the road!", username);
        }

        //TODO: Is there a better way to query whether a file exists?
        if std::fs::metadata("twitter_settings").is_ok() {
            Some(Config {
                token: token,
                user_id: user_id,
                screen_name: username,
            })
        } else {
            None
        }
    }
}

pub mod apiv2_helper {
    use dotenv_codegen::dotenv;
    use eyre::Result;
    use twitter_v2::authorization::{Oauth2Client, Oauth2Token, Scope};
    use twitter_v2::oauth2::basic::BasicTokenType;
    use twitter_v2::oauth2::{
        AuthorizationCode, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier,
        RedirectUrl, RevocationUrl, StandardTokenResponse,
    };
    use twitter_v2::TwitterApi;

    pub fn login() -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        use twitter_v2::oauth2::basic::BasicClient;
        use twitter_v2::oauth2::devicecode::StandardDeviceAuthorizationResponse;
        use twitter_v2::oauth2::reqwest::http_client;
        use twitter_v2::oauth2::{
            AuthUrl, ClientId, ClientSecret, DeviceAuthorizationUrl, Scope, TokenResponse, TokenUrl,
        };
        use url::Url;

        let device_auth_url =
            DeviceAuthorizationUrl::new("https://api.twitter.com/2/device/token".to_string())?;
        let client = BasicClient::new(
            ClientId::new(dotenv!("KEY").to_owned()),
            Some(ClientSecret::new(dotenv!("SECRET").to_owned())),
            AuthUrl::from_url("https://twitter.com/i/oauth2/authorize".parse().unwrap()),
            Some(TokenUrl::from_url(
                "https://api.twitter.com/2/oauth2/token".parse().unwrap(),
            )),
        )
        .set_redirect_uri(RedirectUrl::new("partyboy://redirect".to_string())?)
        .set_revocation_uri(RevocationUrl::from_url(
            "https://api.twitter.com/2/oauth2/revoke".parse().unwrap(),
        ));
        // .set_device_authorization_url(device_auth_url);

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        // This is the URL you should redirect the user to, in order to trigger the authorization
        // process.
        println!("Browse to: {} and enter code:\n", auth_url);

        // Once the user has been redirected to the redirect URL, you'll have access to the
        // authorization code. For security reasons, your code should verify that the `state`
        // parameter returned by the server matches `csrf_state`.

        let mut pin = String::new();
        std::io::stdin().read_line(&mut pin).unwrap();
        println!("");

        // Now you can trade it for an access token.
        let token_result = client
            .exchange_code(AuthorizationCode::new(pin.to_string()))
            // Set the PKCE code verifier.
            .set_pkce_verifier(pkce_verifier)
            .request(http_client)?;

        dbg!(&token_result);

        Ok(token_result)
    }
}
