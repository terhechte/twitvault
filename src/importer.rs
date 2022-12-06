//! If a Twitter archive exists, use it to import tweets and likes
use std::{collections::HashSet, io::Seek, path::Path, str::FromStr, sync::Arc};

use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize};
use tokio::sync::{mpsc::channel, Mutex};
use tracing::{info, warn};

use crate::{config::Config, storage::Storage, types::Message};
use egg_mode::{
    entities::{
        HashtagEntity, MediaEntity, MediaSize, MediaSizes, MentionEntity, UrlEntity, VideoInfo,
        VideoVariant,
    },
    tweet::{ExtendedTweetEntities, Tweet, TweetEntities, TweetSource},
};
use std::borrow::Cow;
use std::io::Read;

const ARCHIVE_DATA_FOLDER: &str = "data";
const ARCHIVE_TWEETS_FILE: &str = "tweets.js";

pub async fn import_archive(
    storage: Storage,
    config: &Config,
    path: impl AsRef<Path>,
) -> Result<Storage> {
    let tweet_file = path
        .as_ref()
        .join(ARCHIVE_DATA_FOLDER)
        .join(ARCHIVE_TWEETS_FILE);
    let mut reader = std::fs::File::open(tweet_file)?;

    // find the beginning of the json
    let buf: &mut [u8] = &mut [0];
    let mut start = 0;
    loop {
        reader.read_exact(buf)?;
        if buf[0] == b'[' && start > 0 {
            reader.seek(std::io::SeekFrom::Start(start - 1))?;
            break;
        }
        start += 1;
    }
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let decoded: Vec<TweetContainer> = serde_json::from_slice(&buffer)?;
    let total_new = decoded.len();

    if total_new == 0 {
        return Ok(storage);
    }

    let known_ids: HashSet<u64> = storage.data().tweets.iter().map(|e| e.id).collect();
    // get a copy of all known tweets so we can insert new ones and in the
    // end sort them all
    let mut tweets = storage.data().tweets.clone();

    let shared_storage = Arc::new(Mutex::new(storage));

    let cloned_storage = shared_storage.clone();
    let (instruction_task, instruction_sender) =
        crate::crawler::create_instruction_handler(config.crawl_options().media, cloned_storage);

    let (message_sender, _) = channel::<Message>(4096);

    // only insert those tweets that we don't have in storage yet.
    // then, collect the profiles and the media
    let mut new_tweets = 0;
    for container in decoded.into_iter() {
        let id = container.tweet.id;
        if known_ids.contains(&id) {
            continue;
        }
        match Tweet::try_from(container.tweet) {
            Ok(n) => {
                if let Err(e) = crate::crawler::inspect_tweet(
                    &n,
                    shared_storage.clone(),
                    config,
                    &instruction_sender,
                    &message_sender,
                )
                .await
                {
                    warn!("Could not inspect tweet {id}: {e:?}");
                }
                tweets.push(n);
                new_tweets += 1;
            }
            Err(e) => {
                warn!("Could not parse tweet {id}: {e:?}");
                continue;
            }
        }
    }

    tweets.sort_by(|a, b| b.id.cmp(&a.id));

    info!("Waiting for Media downloads");
    if let Err(e) = instruction_sender
        .send(crate::crawler::DownloadInstruction::Done)
        .await
    {
        warn!("Could not stop instruction task: {e:?}");
    }

    // wait for the tasks to finish
    info!("Waiting for Instruction task to finish downloading media");
    if let Err(e) = instruction_task.await {
        warn!("Error executing instructions: {e:?}");
    }

    info!("imported {new_tweets} new tweets. Total: {}", tweets.len());

    let mut new_storage = shared_storage.lock_owned().await.clone();

    new_storage.data_mut().tweets = tweets;

    Ok(new_storage)
}

#[derive(Debug, Deserialize)]
struct TweetContainer<'a> {
    #[serde(bound = "'de: 'a")]
    tweet: ArchiveTweet<'a>,
}

#[derive(Debug, Deserialize, Clone)]
struct ArchiveTweet<'a> {
    source: Option<Cow<'a, str>>,
    #[serde(borrow)]
    entities: Entity<'a>,
    #[serde(default, deserialize_with = "deserialize_orange")]
    display_text_range: Option<(usize, usize)>,
    #[serde(deserialize_with = "deserialize_i32")]
    favorite_count: i32,
    truncated: bool,
    #[serde(deserialize_with = "deserialize_i32")]
    retweet_count: i32,
    #[serde(deserialize_with = "deserialize_u64")]
    id: u64,
    extended_entities: Option<ExtendedEntity<'a>>,
    created_at: Cow<'a, str>,
    favorited: Option<bool>,
    full_text: Cow<'a, str>,
    lang: Option<Cow<'a, str>>,
    #[serde(default)]
    in_reply_to_screen_name: Option<Cow<'a, str>>,
    #[serde(default, deserialize_with = "deserialize_ou64")]
    in_reply_to_status_id: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_ou64")]
    in_reply_to_user_id: Option<u64>,
    quoted_status_id: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
struct Entity<'a> {
    #[serde(borrow)]
    hashtags: Vec<ArchiveHashtag<'a>>,
    user_mentions: Vec<ArchiveMention<'a>>,
    urls: Vec<ArchiveUrl<'a>>,
    media: Option<Vec<ArchiveMedia<'a>>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ExtendedEntity<'a> {
    media: Vec<ArchiveMedia<'a>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ArchiveMention<'a> {
    name: Cow<'a, str>,
    screen_name: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_range")]
    indices: (usize, usize),
    #[serde(deserialize_with = "deserialize_u64")]
    id: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct ArchiveUrl<'a> {
    url: Cow<'a, str>,
    expanded_url: Option<Cow<'a, str>>,
    display_url: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_range")]
    indices: (usize, usize),
}

#[derive(Debug, Clone, Deserialize)]
struct ArchiveMedia<'a> {
    display_url: Cow<'a, str>,
    expanded_url: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_u64")]
    id: u64,
    #[serde(deserialize_with = "deserialize_range")]
    indices: (usize, usize),
    media_url: Cow<'a, str>,
    media_url_https: Cow<'a, str>,
    sizes: ArchiveMediaSizes,
    #[serde(rename = "type")]
    media_type: egg_mode::entities::MediaType,
    url: Cow<'a, str>,
    video_info: Option<ArchiveVideoInfo>,
    ext_alt_text: Option<Cow<'a, str>>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct ArchiveMediaSizes {
    thumb: ArchiveMediaSize,
    small: ArchiveMediaSize,
    medium: ArchiveMediaSize,
    large: ArchiveMediaSize,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct ArchiveMediaSize {
    #[serde(deserialize_with = "deserialize_i32")]
    w: i32,
    #[serde(deserialize_with = "deserialize_i32")]
    h: i32,
    resize: egg_mode::entities::ResizeMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ArchiveVideoInfo {
    #[serde(deserialize_with = "deserialize_range")]
    aspect_ratio: (usize, usize),
    #[serde(default, deserialize_with = "deserialize_ou64")]
    duration_millis: Option<u64>,
    variants: Vec<ArchiveVideoVariant>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ArchiveVideoVariant {
    #[serde(default, deserialize_with = "deserialize_ou64")]
    bitrate: Option<u64>,
    #[serde(with = "serde_via_string")]
    content_type: mime::Mime,
    url: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ArchiveHashtag<'a> {
    text: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_range")]
    indices: (usize, usize),
}

impl<'a> TryFrom<ArchiveTweet<'a>> for Tweet {
    type Error = eyre::Error;
    fn try_from(value: ArchiveTweet) -> Result<Self, Self::Error> {
        let created_at = parse_date(&value.created_at)?;
        Ok(Tweet {
            coordinates: None,
            created_at,
            current_user_retweet: None,
            display_text_range: value.display_text_range,
            entities: value.entities.try_into()?,
            extended_entities: value.extended_entities.map(|e| ExtendedTweetEntities {
                media: e
                    .media
                    .into_iter()
                    .flat_map(|o| o.try_into().ok())
                    .collect(),
            }),
            favorite_count: value.favorite_count,
            favorited: value.favorited,
            filter_level: None,
            id: value.id,
            in_reply_to_user_id: value.in_reply_to_user_id,
            in_reply_to_screen_name: value.in_reply_to_screen_name.map(|e| e.to_string()),
            in_reply_to_status_id: value.in_reply_to_status_id,
            lang: value.lang.map(|e| e.to_string()),
            place: None,
            possibly_sensitive: None,
            quoted_status_id: value.quoted_status_id,
            quoted_status: None,
            retweet_count: value.retweet_count,
            retweeted: None,
            retweeted_status: None,
            source: value.source.and_then(|e| TweetSource::from_str(&e).ok()),
            text: value.full_text.to_string(),
            truncated: value.truncated,
            user: None,
            withheld_copyright: false,
            withheld_in_countries: None,
            withheld_scope: None,
        })
    }
}

fn parse_date(date: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    // "Wed Nov 23 08:23:27 +0000 2022"
    use chrono::{DateTime, Utc};
    let date = DateTime::parse_from_str(date, "%a %b %d %H:%M:%S %z %Y")?;
    let utc = date.with_timezone(&Utc);
    Ok(utc)
}

impl<'a> TryFrom<Entity<'a>> for TweetEntities {
    type Error = eyre::Error;

    fn try_from(value: Entity<'a>) -> Result<Self, Self::Error> {
        Ok(TweetEntities {
            hashtags: value
                .hashtags
                .into_iter()
                .flat_map(|e| e.try_into().ok())
                .collect(),
            symbols: Vec::new(),
            urls: value
                .urls
                .into_iter()
                .flat_map(|e| e.try_into().ok())
                .collect(),
            user_mentions: value
                .user_mentions
                .into_iter()
                .flat_map(|e| e.try_into().ok())
                .collect(),
            media: value
                .media
                .map(|v| v.into_iter().flat_map(|e| e.try_into().ok()).collect()),
        })
    }
}

impl<'a> TryFrom<ArchiveHashtag<'a>> for HashtagEntity {
    type Error = eyre::Error;

    fn try_from(value: ArchiveHashtag<'a>) -> Result<Self, Self::Error> {
        Ok(HashtagEntity {
            range: value.indices,
            text: value.text.to_string(),
        })
    }
}

impl<'a> TryFrom<ArchiveMedia<'a>> for MediaEntity {
    type Error = eyre::Error;

    fn try_from(value: ArchiveMedia<'a>) -> Result<Self, Self::Error> {
        Ok(MediaEntity {
            display_url: value.display_url.to_string(),
            expanded_url: value.expanded_url.to_string(),
            id: value.id,
            range: value.indices,
            media_url: value.media_url.to_string(),
            media_url_https: value.media_url_https.to_string(),
            sizes: value.sizes.try_into()?,
            media_type: value.media_type,
            url: value.url.to_string(),
            video_info: value.video_info.and_then(|e| e.try_into().ok()),
            source_status_id: None,
            ext_alt_text: value.ext_alt_text.map(|a| a.to_string()),
        })
    }
}

impl TryFrom<ArchiveMediaSizes> for MediaSizes {
    type Error = eyre::Error;

    fn try_from(value: ArchiveMediaSizes) -> Result<Self, Self::Error> {
        Ok(MediaSizes {
            thumb: value.thumb.try_into()?,
            small: value.small.try_into()?,
            medium: value.medium.try_into()?,
            large: value.large.try_into()?,
        })
    }
}

impl TryFrom<ArchiveMediaSize> for MediaSize {
    type Error = eyre::Error;

    fn try_from(value: ArchiveMediaSize) -> Result<Self, Self::Error> {
        Ok(MediaSize {
            w: value.w,
            h: value.h,
            resize: value.resize,
        })
    }
}

impl TryFrom<ArchiveVideoInfo> for VideoInfo {
    type Error = eyre::Error;

    fn try_from(value: ArchiveVideoInfo) -> Result<Self, Self::Error> {
        let mut variants: Vec<VideoVariant> = Vec::new();
        for f in value.variants {
            if let Ok(n) = f.try_into() {
                variants.push(n);
            }
        }
        Ok(VideoInfo {
            aspect_ratio: (value.aspect_ratio.0 as i32, value.aspect_ratio.1 as i32),
            duration_millis: value.duration_millis.map(|e| e as i32),
            variants,
        })
    }
}

impl TryFrom<ArchiveVideoVariant> for VideoVariant {
    type Error = eyre::Error;

    fn try_from(value: ArchiveVideoVariant) -> Result<Self, Self::Error> {
        Ok(VideoVariant {
            bitrate: value.bitrate.map(|e| e as i32),
            content_type: value.content_type,
            url: value.url,
        })
    }
}

impl<'a> TryFrom<ArchiveUrl<'a>> for UrlEntity {
    type Error = eyre::Error;

    fn try_from(value: ArchiveUrl<'a>) -> Result<Self, Self::Error> {
        Ok(UrlEntity {
            display_url: value.display_url.to_string(),
            expanded_url: value.expanded_url.map(|e| e.to_string()),
            range: value.indices,
            url: value.url.to_string(),
        })
    }
}

impl<'a> TryFrom<ArchiveMention<'a>> for MentionEntity {
    type Error = eyre::Error;

    fn try_from(value: ArchiveMention<'a>) -> Result<Self, Self::Error> {
        Ok(MentionEntity {
            id: value.id,
            range: value.indices,
            name: value.name.to_string(),
            screen_name: value.screen_name.to_string(),
        })
    }
}

fn deserialize_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let number = buf.parse::<i32>().map_err(serde::de::Error::custom)?;
    Ok(number)
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    // deleted users are -1
    if buf == "-1" {
        return Ok(0);
    }
    let number = buf.parse::<u64>().map_err(serde::de::Error::custom)?;
    Ok(number)
}

fn deserialize_ou64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(buf) = String::deserialize(deserializer).ok() else {
        return Ok(None)
    };
    let number = buf.parse::<u64>().map_err(serde::de::Error::custom)?;
    Ok(Some(number))
}

fn deserialize_range<'de, D>(deserializer: D) -> Result<(usize, usize), D::Error>
where
    D: Deserializer<'de>,
{
    let data: Vec<&'de str> = Vec::deserialize(deserializer)?;
    let start = data[0].parse::<usize>().map_err(serde::de::Error::custom)?;
    let end = data[1].parse::<usize>().map_err(serde::de::Error::custom)?;
    Ok((start, end))
}

fn deserialize_orange<'de, D>(deserializer: D) -> Result<Option<(usize, usize)>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(data) = Vec::<&'de str>::deserialize(deserializer).ok() else {
        return Ok(None)
    };
    let start = data[0].parse::<usize>().map_err(serde::de::Error::custom)?;
    let end = data[1].parse::<usize>().map_err(serde::de::Error::custom)?;
    Ok(Some((start, end)))
}

pub mod serde_via_string {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    use std::fmt;

    pub fn deserialize<'de, D, T>(ser: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let str = String::deserialize(ser)?;
        str.parse().map_err(D::Error::custom)
    }

    pub fn serialize<T, S>(src: &T, ser: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        ser.collect_str(src)
    }
}
