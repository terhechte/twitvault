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

use super::loaded_component::{ColumnState, COLUMN2};
use super::tweet_list::TweetListComponent;
use super::types::StorageWrapper;
use super::user_component::AuthorComponent;

#[inline_props]
pub fn SecondaryColumn(
    cx: Scope,
    storage: StorageWrapper,
    selected: AtomState<ColumnState>,
) -> Element {
    let column2 = use_atom_state(&cx, COLUMN2);

    if column2.current().as_ref() == &ColumnState::None {
        return cx.render(rsx! { div { }});
    }

    let column_class = "d-flex flex-column flex-shrink-0 bg-light";
    let column_style = "width: 35rem; overflow: scroll; padding: 12px; height: 100vh;";

    let column = cx.render(rsx!(div {
        {if let ColumnState::Responses(id) = column2.current().as_ref() {
            let label = "Responses".to_string();
            rsx!{
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    TweetListComponent {
                        data: storage.data().responses.get(id).unwrap_or(&storage.empty_tweets),
                        media: &storage.data().media,
                        label: label,
                        user: &storage.data().profile,
                        responses: &storage.data().responses
                    }
                }
            }
        } else {rsx!{ div {} }}}

        {if let ColumnState::Profile(id) = column2.current().as_ref() {
            if let Some(profile) = storage.data().profiles.get(id) {
                rsx!{
                    div {
                        class: "{column_class}",
                        style: "{column_style}",
                        AuthorComponent {
                            profile: profile,
                            media: &storage.data().media
                        }
                    }
                }
            } else {
                rsx! { div {
                    "Profile {id} not found"
                }}
            }
        } else {rsx!{ div {} }}}

    }));

    cx.render(rsx! {div {
        class: "d-grid gap-2",
        button {
            r#type: "button",
            class: "btn btn-secondary",
            onclick: move |_| selected.set(ColumnState::None),
            "Close"
        }
        column
    }})
}
