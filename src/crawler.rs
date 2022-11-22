use crate::storage::{List, Storage};
use crate::types::Message;
use egg_mode::Token;
use egg_mode::{
    cursor,
    list::{self, ListID},
    tweet::{self, Tweet},
    user::{self, TwitterUser},
    RateLimit,
};
use reqwest::Client;
use std::io::Write;
use std::{collections::HashSet, path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::{
    mpsc::{channel, Sender},
    Mutex,
};
use tracing::{info, trace, warn};

use eyre::Result;

use crate::helpers::Config;

/// Internal messaging between the different threads
#[derive(Debug)]
enum DownloadInstruction {
    Image(String),
    Movie(mime::Mime, String),
    ProfileMedia(String),
    Done,
}

pub async fn fetch(config: &Config, storage: Storage, sender: Sender<Message>) -> Result<()> {
    let user_id = storage.data().profile.id;
    let shared_storage = Arc::new(Mutex::new(storage));

    async fn msg(msg: impl AsRef<str>, sender: &Sender<Message>) -> Result<()> {
        Ok(sender
            .send(Message::Loading(msg.as_ref().to_string()))
            .await?)
    }

    async fn save_data(storage: &Arc<Mutex<Storage>>) {
        if let Err(e) = storage.lock().await.save() {
            warn!("Could not write out data {e:?}");
        }
    }

    let (instruction_sender, mut instruction_receiver) = channel(128);
    let cloned_storage = shared_storage.clone();
    let instruction_task = tokio::spawn(async move {
        let client = Client::new();
        while let Some(instruction) = instruction_receiver.recv().await {
            if matches!(instruction, DownloadInstruction::Done) {
                break;
            }
            if let Err(e) = handle_instruction(&client, instruction, cloned_storage.clone()).await {
                warn!("Download Error {e:?}");
            }
        }
    });

    // msg("User Tweets", &sender).await?;
    // fetch_user_tweets(
    //     user_id,
    //     shared_storage.clone(),
    //     config,
    //     instruction_sender.clone(),
    // )
    // .await?;
    // save_data(&shared_storage).await;

    // msg("User Mentions", &sender).await?;
    // fetch_user_mentions(shared_storage.clone(), config, instruction_sender.clone()).await?;
    // save_data(&shared_storage).await;

    // msg("Followers", &sender).await?;
    // fetch_user_followers(
    //     user_id,
    //     shared_storage.clone(),
    //     config,
    //     instruction_sender.clone(),
    // )
    // .await?;
    // save_data(&shared_storage).await;

    // msg("Follows", &sender).await?;
    // fetch_user_follows(
    //     user_id,
    //     shared_storage.clone(),
    //     config,
    //     instruction_sender.clone(),
    // )
    // .await?;
    // save_data(&shared_storage).await;

    // msg("Lists", &sender).await?;
    // fetch_lists(
    //     user_id,
    //     shared_storage.clone(),
    //     config,
    //     instruction_sender.clone(),
    // )
    // .await?;
    // save_data(&shared_storage).await;

    msg("Bookmarks", &sender).await?;
    fetch_bookmarks(shared_storage.clone(), config, &instruction_sender).await?;
    save_data(&shared_storage).await;

    instruction_sender.send(DownloadInstruction::Done);
    instruction_task.await?;

    let storage = shared_storage.lock_owned().await.clone();
    sender.send(Message::Finished(storage));

    Ok(())
}

async fn fetch_user_tweets(
    id: u64,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    let mut timeline = tweet::user_timeline(id, true, true, &config.token).with_page_size(50);

    loop {
        tracing::info!("Downloading Tweets before {:?}", timeline.min_id);
        let (next_timeline, mut feed) = timeline.older(None).await?;
        if feed.response.is_empty() {
            break;
        }
        for tweet in feed.response.iter() {
            inspect_tweet(tweet, shared_storage.clone(), config, &sender).await?;
        }
        shared_storage
            .lock()
            .await
            .data_mut()
            .tweets
            .append(&mut feed.response);

        handle_rate_limit(&feed.rate_limit_status, "User Feed").await;
        timeline = next_timeline;
    }

    Ok(())
}

async fn fetch_user_mentions(
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    let mut timeline = tweet::mentions_timeline(&config.token).with_page_size(50);

    loop {
        tracing::info!("Downloading Mentions before {:?}", timeline.min_id);
        let (next_timeline, mut feed) = timeline.older(None).await?;
        if feed.response.is_empty() {
            break;
        }
        for tweet in feed.response.iter() {
            inspect_tweet(tweet, shared_storage.clone(), config, &sender).await?;
        }
        shared_storage
            .lock()
            .await
            .data_mut()
            .mentions
            .append(&mut feed.response);

        handle_rate_limit(&feed.rate_limit_status, "User Mentions").await;
        timeline = next_timeline;
    }

    Ok(())
}

async fn fetch_user_followers(
    id: u64,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    let ids = fetch_profiles_ids(
        user::followers_ids(id, &config.token).with_page_size(50),
        shared_storage.clone(),
        config,
        sender,
    )
    .await?;
    shared_storage.lock().await.data_mut().followers = ids;
    Ok(())
}

async fn fetch_user_follows(
    id: u64,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    let ids = fetch_profiles_ids(
        user::friends_ids(id, &config.token).with_page_size(50),
        shared_storage.clone(),
        config,
        sender,
    )
    .await?;
    shared_storage.lock().await.data_mut().follows = ids;
    Ok(())
}

// Helpers

async fn fetch_profiles_ids(
    mut cursor: cursor::CursorIter<cursor::IDCursor>,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<Vec<u64>> {
    let mut ids = Vec::new();
    loop {
        let called = cursor.call();
        let resp = match called.await {
            Ok(n) => n,
            Err(e) => {
                warn!("Profile Ids Error {e:?}");
                continue;
            }
        };

        let mut new_ids = resp.response.ids.clone();

        if new_ids.is_empty() {
            break;
        }

        fetch_multiple_profiles_data(&new_ids, shared_storage.clone(), config, sender.clone())
            .await?;

        ids.append(&mut new_ids);

        handle_rate_limit(&resp.rate_limit_status, "Follows / Followers").await;
        cursor.next_cursor = resp.response.next_cursor;
    }

    Ok(ids)
}

async fn fetch_multiple_profiles_data(
    ids: &[u64],
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    // only get profiles we haven't gotten yet
    let known_ids: HashSet<u64> = shared_storage
        .lock()
        .await
        .data()
        .profiles
        .keys()
        .copied()
        .collect();
    let filtered: Vec<_> = ids
        .iter()
        .filter(|id| !known_ids.contains(id))
        .copied()
        .collect();
    let profiles = user::lookup(filtered, &config.token).await?;
    for profile in profiles.iter() {
        inspect_profile(profile, sender.clone()).await?;
    }
    shared_storage.lock().await.with_data(move |data| {
        for profile in &profiles.response {
            data.profiles.insert(profile.id, profile.clone());
        }
    });
    Ok(())
}

async fn fetch_lists(
    id: u64,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    let mut cursor = list::ownerships(id, &config.token).with_page_size(500);
    loop {
        let called = cursor.call();
        let resp = match called.await {
            Ok(n) => n,
            Err(e) => {
                warn!("Lists Error {e:?}");
                continue;
            }
        };

        let lists = resp.response.lists;

        if lists.is_empty() {
            break;
        }

        for list in lists {
            fetch_list_members(list, shared_storage.clone(), config, sender.clone()).await?;
        }

        handle_rate_limit(&resp.rate_limit_status, "Lists").await;
        cursor.next_cursor = resp.response.next_cursor;
    }
    Ok(())
}

async fn fetch_list_members(
    list: list::List,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    let list_id = ListID::from_id(list.id);
    let mut cursor = list::members(list_id, &config.token).with_page_size(2000);
    let mut member_ids = Vec::new();
    loop {
        let called = cursor.call();
        let resp = match called.await {
            Ok(n) => n,
            Err(e) => {
                warn!("List Members Error {e:?}");
                continue;
            }
        };

        if resp.users.is_empty() {
            break;
        }

        let mut storage = shared_storage.lock().await;

        for member in &resp.users {
            if let Err(e) = inspect_profile(member, sender.clone()).await {
                warn!("Could not inspect profile {e:?}");
            }
            member_ids.push(member.id);
            storage
                .data_mut()
                .profiles
                .insert(member.id, member.clone());
        }

        handle_rate_limit(&resp.rate_limit_status, "List Members").await;
        cursor.next_cursor = resp.response.next_cursor;
    }

    shared_storage.lock().await.data_mut().lists.push(List {
        name: list.name.clone(),
        list,
        members: member_ids,
    });

    Ok(())
}

async fn fetch_single_profile(
    id: u64,
    shared_storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: Sender<DownloadInstruction>,
) -> Result<()> {
    if shared_storage
        .lock()
        .await
        .data()
        .profiles
        .contains_key(&id)
    {
        return Ok(());
    }

    let user = user::show(id, &config.token).await?;
    if let Err(e) = inspect_profile(&user, sender).await {
        warn!("Inspect profile error {e:?}");
    }

    shared_storage
        .lock()
        .await
        .data_mut()
        .profiles
        .insert(id, user.response);
    Ok(())
}

async fn inspect_tweet(
    tweet: &Tweet,
    storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: &Sender<DownloadInstruction>,
) -> Result<()> {
    if let Err(e) = inspect_inner_tweet(tweet, sender.clone()).await {
        warn!("Inspect Tweet Error {e:?}");
    }

    if let Some(quoted_tweet) = &tweet.quoted_status {
        if let Err(e) = inspect_inner_tweet(quoted_tweet, sender.clone()).await {
            warn!("Inspect Quoted Tweet Error {e:?}");
        }
    }

    if let Some(retweet) = &tweet.retweeted_status {
        if let Err(e) = inspect_inner_tweet(retweet, sender.clone()).await {
            warn!("Inspect Retweet Error {e:?}");
        }
    }

    if let Some(user) = &tweet.user {
        if let Err(e) = fetch_single_profile(user.id, storage.clone(), config, sender.clone()).await
        {
            warn!("Could not download profile {e:?}");
        }
    }

    Ok(())
}

async fn inspect_inner_tweet(tweet: &Tweet, sender: Sender<DownloadInstruction>) -> Result<()> {
    let Some(entities) = &tweet.extended_entities else { return Ok(()) };

    for media in &entities.media {
        match &media.video_info {
            Some(n) => {
                let mut selected_variant = n.variants.first();
                for variant in &n.variants {
                    match (
                        variant.content_type.subtype(),
                        &selected_variant.map(|e| e.bitrate),
                    ) {
                        (mime::MP4, Some(bitrate)) if bitrate > &variant.bitrate => {
                            selected_variant = Some(variant)
                        }
                        _ => (),
                    }
                }
                let Some(variant) = selected_variant else { continue };
                if let Err(e) = sender
                    .send(DownloadInstruction::Movie(
                        variant.content_type.clone(),
                        variant.url.clone(),
                    ))
                    .await
                {
                    warn!("Send Error {e:?}");
                }
            }
            None => {
                if let Err(e) = sender
                    .send(DownloadInstruction::Image(media.media_url_https.clone()))
                    .await
                {
                    warn!("Send Error {e:?}");
                }
            }
        }
    }
    Ok(())
}

async fn inspect_profile(profile: &TwitterUser, sender: Sender<DownloadInstruction>) -> Result<()> {
    if let Some(background_image) = profile.profile_background_image_url_https.as_ref() {
        sender
            .send(DownloadInstruction::ProfileMedia(background_image.clone()))
            .await?;
    }
    if let Some(profile_banner_url) = profile.profile_banner_url.as_ref() {
        sender
            .send(DownloadInstruction::ProfileMedia(
                profile_banner_url.clone(),
            ))
            .await?;
    }
    sender
        .send(DownloadInstruction::ProfileMedia(
            profile.profile_image_url_https.clone(),
        ))
        .await?;
    Ok(())
}

async fn handle_instruction(
    client: &Client,
    instruction: DownloadInstruction,
    shared_storage: Arc<Mutex<Storage>>,
) -> Result<()> {
    let (extension, url) = match instruction {
        DownloadInstruction::Image(url) => (extension_for_url(&url), url),
        DownloadInstruction::Movie(mime, url) => (extension_for_url(&url), url),
        DownloadInstruction::ProfileMedia(url) => (extension_for_url(&url), url),
        _ => return Ok(()),
    };
    let path = {
        let storage = shared_storage.lock().await;
        if storage.data().media.contains_key(&url) {
            return Ok(());
        }

        let file_stem = uuid::Uuid::new_v4().to_string();
        let file_name = format!("{file_stem}.{extension}");
        storage.media_path(&file_name)
    };
    let mut fp = std::fs::File::create(&path)?;

    trace!("Downloading {url} into {}", path.display());

    let bytes = client.get(url).send().await?.bytes().await?;

    fp.write_all(&bytes)?;

    Ok(())
}

fn extension_for_url(url: &str) -> String {
    let default = "png".to_string();
    let Ok(parsed) = url::Url::parse(url) else {
        return default;
    };
    let Some(Some(last_part)) = parsed.path_segments().and_then(|e| e.last().map(|p| PathBuf::from_str(p).ok())) else {
        return default;
    };
    let Some(extension) = last_part.extension().and_then(|e| e.to_str().map(|s| s.to_string())) else {
        return default
    };
    extension
}

/// If the rate limit for a call is used up, delay that particular call
async fn handle_rate_limit(limit: &RateLimit, call_info: &'static str) {
    if limit.remaining <= 1 {
        let seconds = {
            use std::time::{SystemTime, UNIX_EPOCH};
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => (((limit.reset as i64) - n.as_secs() as i64) + 10) as u64,
                Err(_) => 1000,
            }
        };
        info!("Rate limit for {call_info} reached. Waiting {seconds} seconds");
        let wait_duration = tokio::time::Duration::from_secs(seconds);
        tokio::time::sleep(wait_duration).await;
    } else {
        trace!(
            "Rate limit for {call_info}: {} / {}",
            limit.remaining,
            limit.limit
        );
    }
}

/// Bookmarks require the V2 API, therefore they're a bit awful here
async fn fetch_bookmarks(
    storage: Arc<Mutex<Storage>>,
    config: &Config,
    sender: &Sender<DownloadInstruction>,
) -> Result<()> {
    use twitter_v2::authorization::Oauth1aToken;
    use twitter_v2::TwitterApi;
    let Token::Access { consumer, access } = &config.token else {
        warn!("Invalid token type for Twitter API V2");
        return Ok(());
    };
    // let consumer_key = dotenv!("KEY");
    // let consumer_secret = dotenv!("SECRET");
    let auth = Oauth1aToken::new(
        consumer.key.clone(),
        consumer.secret.clone(),
        access.key.clone(),
        access.secret.clone(),
    );

    let api = TwitterApi::new(auth);
    let mut next_token: Option<String> = None;

    let mut bookmarks = Vec::new();

    loop {
        let response = api
            .get_user_bookmarks(config.user_id)
            .max_results(100)
            .pagination_token(next_token.clone().unwrap_or_default().as_str())
            .send()
            .await?;
        next_token = response.meta().and_then(|e| e.next_token.clone());
        let Some(data) = response.into_data() else {
            break;
        };
        for tweet in data {
            let tweet = conversion::convert_tweet(&tweet);
            if let Err(e) = inspect_tweet(&tweet, storage.clone(), &config, &sender).await {
                warn!("Could not inspect {e:?}");
            }
            bookmarks.push(tweet);
        }
    }

    storage.lock().await.data_mut().bookmarks = bookmarks;

    // .meta

    Ok(())
}

mod conversion {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use egg_mode::{
        entities::UrlEntity,
        tweet::{Tweet, TweetEntities, TweetSource},
    };
    use time::OffsetDateTime;
    use twitter_v2::data::FullTextEntities;

    pub fn convert_tweet(tweet: &twitter_v2::Tweet) -> egg_mode::tweet::Tweet {
        Tweet {
            coordinates: None,
            created_at: convert_time(tweet.created_at),
            current_user_retweet: None,
            display_text_range: None,
            entities: convert_entities(tweet.entities.as_ref()),
            extended_entities: None,
            favorite_count: tweet
                .organic_metrics
                .as_ref()
                .map(|e| e.like_count as i32)
                .unwrap_or_default(),
            favorited: None,
            filter_level: None,
            id: tweet.id.as_u64(),
            in_reply_to_user_id: tweet.in_reply_to_user_id.map(|e| e.as_u64()),
            in_reply_to_screen_name: None,
            in_reply_to_status_id: None,
            lang: tweet.lang.clone(),
            place: None,
            possibly_sensitive: tweet.possibly_sensitive,
            quoted_status_id: tweet
                .referenced_tweets
                .as_ref()
                .map(|e| e.first().map(|o| o.id.as_u64()))
                .flatten(),
            quoted_status: None,
            retweet_count: tweet
                .organic_metrics
                .as_ref()
                .map(|e| e.retweet_count as i32)
                .unwrap_or_default(),
            retweeted: None,
            retweeted_status: None,
            source: tweet.source.as_ref().map(|e| TweetSource {
                name: e.clone(),
                url: "".to_owned(),
            }),
            text: tweet.text.clone(),
            truncated: false,
            user: None,
            withheld_copyright: false,
            withheld_in_countries: None,
            withheld_scope: None,
        }
    }

    fn convert_time(time: Option<OffsetDateTime>) -> chrono::DateTime<chrono::Utc> {
        let Some(time) = time else {
            return Utc::now();
        };
        // Create a NaiveDateTime from the timestamp
        let naive = NaiveDateTime::from_timestamp_opt(time.unix_timestamp(), 0).unwrap();

        // Create a normal DateTime from the NaiveDateTime
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        datetime
    }

    fn convert_entities(entities: Option<&FullTextEntities>) -> TweetEntities {
        let Some(entities) = entities else {
        return TweetEntities {
            hashtags: Vec::new(),
            symbols: Vec::new(),
            urls: Vec::new(),
            user_mentions: Vec::new(),
            media: None,
        }
        };
        let urls = entities
            .urls
            .clone()
            .unwrap_or_default()
            .iter()
            .map(convert_url_entity)
            .collect();
        let hashtags = entities
            .hashtags
            .clone()
            .unwrap_or_default()
            .iter()
            .map(convert_hashtag_entity)
            .collect();
        let user_mentions = entities
            .mentions
            .clone()
            .unwrap_or_default()
            .iter()
            .map(convert_mention_entity)
            .collect();
        TweetEntities {
            hashtags,
            symbols: Vec::new(),
            urls,
            user_mentions,
            media: None,
        }
    }

    fn convert_url_entity(url: &twitter_v2::data::UrlEntity) -> UrlEntity {
        UrlEntity {
            display_url: url.display_url.clone(),
            expanded_url: Some(url.expanded_url.clone()),
            range: (url.start, url.end),
            url: url.url.clone(),
        }
    }

    fn convert_hashtag_entity(
        hashtag: &twitter_v2::data::HashtagEntity,
    ) -> egg_mode::entities::HashtagEntity {
        egg_mode::entities::HashtagEntity {
            range: (hashtag.start, hashtag.end),
            text: hashtag.tag.clone(),
        }
    }

    fn convert_mention_entity(
        mention: &twitter_v2::data::MentionEntity,
    ) -> egg_mode::entities::MentionEntity {
        egg_mode::entities::MentionEntity {
            id: mention.id.map(|m| m.as_u64()).unwrap_or_default(),
            range: (mention.start, mention.end),
            name: mention.username.clone(),
            screen_name: mention.username.clone(),
        }
    }
}
