#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf};

use dioxus::prelude::*;
use egg_mode::user::TwitterUser;

use crate::storage::UrlString;

use egg_mode::tweet::Tweet;

use super::helpers::{BottomSpacer, ShowMoreButton};
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
    let page_size = 100;
    let page = use_state(&cx, || page_size);
    let has_more = cx.props.data.len() > *page.get();
    let tweets_rendered = cx.props.data.iter().take(*page.get()).map(|tweet| {
        let responses = cx.props.responses.get(&tweet.id).as_ref().map(|e| e.len());
        cx.render(rsx!(TweetComponent {
            tweet: tweet,
            media: cx.props.media,
            user: cx.props.user
            responses: responses
        }))
    });

    cx.render(rsx!(div {
        onscroll: |evt| {
            dbg!(evt);
        },
        h5 { "{cx.props.label}" }
        tweets_rendered
        ShowMoreButton {
            visible: has_more,
            onclick: move |_| page.set(page.get() + page_size)
        }
        BottomSpacer {}
    }
    ))
}
