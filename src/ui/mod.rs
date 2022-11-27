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

mod app;
mod loaded_component;
mod loading_component;
mod login_component;
mod primary_column;
mod secondary_column;
mod setup_component;
mod tweet_component;
mod tweet_list;
mod types;
mod user_component;
mod user_list;

pub use app::run_ui;

#[inline_props]
fn ErrorComponent(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            h1 { "Error" }
        }
    })
}
