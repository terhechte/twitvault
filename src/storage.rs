use egg_mode::{list, tweet::Tweet, user::TwitterUser};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// The folder locations for the different data
const FOLDER_MEDIA: &str = "media";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub name: String,
    pub list: list::List,
    pub members: Vec<UserId>,
}

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
    /// Bookmarked posts FIXME: Needs the v2 api, can I re-use the token?
    pub bookmarks: Vec<Tweet>,
    /// Lists
    pub lists: Vec<List>,
    /// Downloaded media with path to local file
    /// - Tweet Media: ExtendedUrlString
    /// - Profiles: Various Urls
    pub media: HashMap<UrlString, PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Storage {
    root_folder: PathBuf,
    data_path: PathBuf,
    data: Data,
}

impl Storage {
    pub fn media_path(&self, filename: &str) -> PathBuf {
        self.root_folder.join(FOLDER_MEDIA).join(filename)
    }

    pub fn new(profile: TwitterUser, path: impl AsRef<Path>) -> Self {
        let root_folder = path.as_ref().to_path_buf();
        if !root_folder.exists() {
            std::fs::create_dir(&root_folder).expect("Can't crate media folder");
        }
        if !root_folder.join(FOLDER_MEDIA).exists() {
            std::fs::create_dir(&root_folder.join(FOLDER_MEDIA)).expect("Can't crate media folder");
        }
        let data_path = root_folder.join(format!("{}.json", profile.id));
        Storage {
            root_folder,
            data_path,
            data: Data {
                profile,
                tweets: Default::default(),
                mentions: Default::default(),
                responses: Default::default(),
                profiles: Default::default(),
                followers: Default::default(),
                follows: Default::default(),
                bookmarks: Default::default(),
                lists: Default::default(),
                media: Default::default(),
            },
        }
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    // pub fn open(path: impl AsRef<Path>) -> Result<Self> {
    //     let input = std::fs::read(&path)?;
    //     let data: Data = serde_json::from_slice(&input)?;
    //     Ok(Storage {
    //         path: path.as_ref().to_path_buf(),
    //         data,
    //     })
    // }

    pub fn with_data(&mut self, action: impl Fn(&mut Data)) {
        action(&mut self.data)
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

impl Storage {
    // fn
}
