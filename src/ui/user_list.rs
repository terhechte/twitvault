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

use super::user_component::AuthorComponent;

#[derive(Props)]
pub struct AuthorListProps<'a> {
    data: &'a [u64],
    media: &'a HashMap<UrlString, PathBuf>,
    profiles: &'a HashMap<u64, TwitterUser>,
    label: String,
}

pub fn AuthorListComponent<'a>(cx: Scope<'a, AuthorListProps>) -> Element<'a> {
    let profiles_rendered = cx.props.data.iter().map(|id| {
        if let Some(user) = cx.props.profiles.get(id) {
            cx.render(rsx!(AuthorComponent {
                profile: user,
                media: cx.props.media,
            }))
        } else {
            cx.render(rsx!(div {
                "Could not find profile {id}"
            }))
        }
    });

    cx.render(rsx!(div {
        h5 { "{cx.props.label}" }
        profiles_rendered
    }
    ))
}
