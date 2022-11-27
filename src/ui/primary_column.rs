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

use super::loaded_component::Tab;
use super::tweet_list::TweetListComponent;
use super::types::StorageWrapper;
use super::user_list::AuthorListComponent;

#[inline_props]
pub fn MainColumn(cx: Scope, storage: StorageWrapper, selected: UseState<Tab>) -> Element {
    let current = (*selected.current()).clone();
    let label = current.to_string();

    let column_class = "d-flex flex-column flex-shrink-0 bg-light";
    let column_style = "width: 35rem; overflow: scroll; padding: 12px; height: 100vh;";

    cx.render(rsx!(div {
        {if current == Tab::Tweets {
            let label = label.clone();
            rsx!{
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    TweetListComponent {
                        data: &storage.data().tweets,
                        media: &storage.data().media,
                        label: label,
                        user: &storage.data().profile,
                        responses: &storage.data().responses
                    }
                }
            }
        } else {rsx!{ div {} }}}
        {if current == Tab::Mentions {
            let label = current.to_string();
            rsx!{
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    TweetListComponent {
                        data: &storage.data().mentions,
                        media: &storage.data().media,
                        label: label.clone(),
                        user: &storage.data().profile,
                        responses: &storage.data().responses
                    }
                }
            }
        } else {rsx!{ div { }}}}
        {if current == Tab::Follows {
            let label = label.clone();
            rsx! {
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    AuthorListComponent {
                        data: &storage.data().follows
                        media: &storage.data().media,
                        profiles: &storage.data().profiles,
                        label: label.clone(),
                    }
                }
            }
        } else {rsx!{ div {}}}}
        {if current == Tab::Followers {
            let label = label.clone();
            rsx! {
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    AuthorListComponent {
                        data: &storage.data().followers
                        media: &storage.data().media,
                        profiles: &storage.data().profiles,
                        label: label.clone(),
                    }
                }
            }
        } else {rsx!{ div {}}}}
    }))
}
