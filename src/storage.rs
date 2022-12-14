use egg_mode::{list, tweet::Tweet, user::TwitterUser};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// The folder locations for the different data
const FOLDER_MEDIA: &str = "media";
const FILE_ROOT: &str = "_data.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub name: String,
    pub list: list::List,
    pub members: Vec<UserId>,
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.list.id == other.list.id
    }
}

impl Eq for List {}

pub type UserId = u64;
pub type TweetId = u64;
pub type UrlString = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    /// The profile of the owner
    pub profile: TwitterUser,
    /// The tweets of the owner
    pub tweets: Vec<Tweet>,
    /// Mentions of the owner
    pub mentions: Vec<Tweet>,
    /// Responses to tweets of the owner: FIXME: Not ther eyet
    pub responses: HashMap<TweetId, Vec<Tweet>>,
    /// Profiles from responses, bookmarks, DMs,
    /// followers and follows
    pub profiles: HashMap<UserId, TwitterUser>,
    /// Followers
    pub followers: Vec<UserId>,
    /// Follows
    pub follows: Vec<UserId>,
    /// Lists
    pub lists: Vec<List>,
    /// Downloaded media with path to local file
    /// - Tweet Media: ExtendedUrlString
    /// - Profiles: Various Urls
    pub media: HashMap<UrlString, String>,
    /// The likes the user performed
    #[serde(default)]
    pub likes: Vec<Tweet>,
}

impl Data {
    pub fn any_tweet(&self, id: TweetId) -> Option<&Tweet> {
        for tweets in [&self.tweets, &self.mentions, &self.likes] {
            for t in tweets {
                if t.id == id {
                    return Some(t);
                }
            }
        }
        for tweets in self.responses.values() {
            for t in tweets {
                if t.id == id {
                    return Some(t);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Storage {
    pub root_folder: PathBuf,
    data_path: PathBuf,
    data: Data,
}

impl Storage {
    fn storage_for_data(path: impl AsRef<Path>, data: Data) -> Result<Self> {
        let root_folder = path.as_ref().to_path_buf();
        if !root_folder.exists() {
            std::fs::create_dir(&root_folder)?;
        }
        if !root_folder.join(FOLDER_MEDIA).exists() {
            std::fs::create_dir(&root_folder.join(FOLDER_MEDIA))?;
        }
        let data_path = root_folder.join(FILE_ROOT);
        Ok(Storage {
            root_folder,
            data_path,
            data,
        })
    }

    pub fn media_path(&self, filename: &str) -> PathBuf {
        self.root_folder.join(FOLDER_MEDIA).join(filename)
    }

    pub fn new(profile: TwitterUser, path: impl AsRef<Path>) -> Result<Self> {
        Self::storage_for_data(
            path,
            Data {
                profile,
                tweets: Default::default(),
                mentions: Default::default(),
                responses: Default::default(),
                profiles: Default::default(),
                followers: Default::default(),
                follows: Default::default(),
                lists: Default::default(),
                media: Default::default(),
                likes: Default::default(),
            },
        )
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let data_path = path.as_ref().join(FILE_ROOT);
        let input = std::fs::read(&data_path)?;
        let data: Data = serde_json::from_slice(&input)?;
        Self::storage_for_data(path, data)
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn with_data(&mut self, action: impl Fn(&mut Data)) {
        action(&mut self.data)
    }

    pub fn resolver(&self) -> MediaResolver {
        MediaResolver {
            root_folder: self.root_folder.join(FOLDER_MEDIA),
            media: &self.data.media,
        }
    }

    // Blocking write
    pub fn save(&self) -> Result<()> {
        use std::fs::OpenOptions;
        let outfile = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.data_path)?;
        Ok(serde_json::to_writer(outfile, &self.data)?)
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct MediaResolver<'a> {
    root_folder: PathBuf,
    media: &'a HashMap<UrlString, String>,
}

impl<'a> MediaResolver<'a> {
    pub fn resolve(&self, url: &str) -> Option<String> {
        // if we're on windows, we just return the URL. Somehow the file locating
        // trick we use with Dioxus doesn't work on Windows
        #[cfg(target_os = "windows")]
        {
            Some(url.to_string())
        }

        #[cfg(not(target_os = "windows"))]
        {
            let found = self.media.get(url)?;
            let path = self.root_folder.join(found);
            Some(path.display().to_string())
        }
    }
}
