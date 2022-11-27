#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf, rc::Rc};

use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::events::*;
use dioxus::fermi::{use_atom_state, AtomState};
use dioxus::prelude::*;
use egg_mode::user::TwitterUser;
use tokio::sync::mpsc::channel;
use tracing::warn;

use crate::config::{Config, RequestData};
use crate::crawler::DownloadInstruction;
use crate::storage::{Data, Storage, TweetId, UrlString, UserId};
use crate::types::Message;
use egg_mode::tweet::Tweet;

#[derive(Clone)]
pub enum LoadingState {
    Login,
    Setup(Config),
    Loading(Config),
    Loaded(StorageWrapper),
}

impl PartialEq for LoadingState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Setup(_), Self::Setup(_)) => true,
            (Self::Loading(_), Self::Loading(_)) => true,
            (Self::Loaded(_), Self::Loaded(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Clone)]
pub struct StorageWrapper {
    data: Rc<Storage>,
    pub empty_tweets: Vec<Tweet>,
}

impl StorageWrapper {
    pub fn new(storage: Storage) -> Self {
        Self {
            data: Rc::new(storage),
            empty_tweets: Vec::new(),
        }
    }

    pub fn data(&self) -> &Data {
        self.data.data()
    }
}

impl PartialEq for StorageWrapper {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for StorageWrapper {}

impl Default for LoadingState {
    fn default() -> Self {
        // TEMPORARY
        //let data = Storage::open("archive_terhechte").unwrap();

        // let s = Config::archive_path();
        // let data = Storage::open(s).unwrap();
        // LoadingState::Loaded(StorageWrapper::new(data))
        LoadingState::Login
    }
}
