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

use super::tweet_component::TweetComponent;

#[derive(Props)]
pub struct TweetListProps<'a> {
    data: &'a [Tweet],
    media: &'a HashMap<UrlString, PathBuf>,
    user: &'a TwitterUser,
    responses: &'a HashMap<u64, Vec<Tweet>>,
    label: String,
}

pub fn TweetListComponent<'a>(cx: Scope<'a, TweetListProps>) -> Element<'a> {
    let tweets_rendered = cx.props.data.iter().map(|tweet| {
        let responses = cx.props.responses.get(&tweet.id).as_ref().map(|e| e.len());
        cx.render(rsx!(TweetComponent {
            tweet: tweet,
            media: cx.props.media,
            user: cx.props.user
            responses: responses
        }))
    });

    cx.render(rsx!(div {
        h5 { "{cx.props.label}" }
        tweets_rendered
    }
    ))
}
