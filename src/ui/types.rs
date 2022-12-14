#![allow(non_snake_case)]
use std::rc::Rc;

use crate::config::Config;

use crate::storage::{Data, MediaResolver, Storage};

use egg_mode::tweet::Tweet;

#[derive(Clone)]
pub enum LoadingState {
    Login,
    Setup(Config),
    Loading(Config),
    Loaded(StorageWrapper, Config),
}

impl PartialEq for LoadingState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Setup(_), Self::Setup(_)) => true,
            (Self::Loading(_), Self::Loading(_)) => true,
            (Self::Loaded(_, _), Self::Loaded(_, _)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for LoadingState {}

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

    pub fn resolver(&self) -> MediaResolver {
        self.data.resolver()
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
        LoadingState::Login
    }
}
