#![allow(non_snake_case)]
use std::collections::HashMap;

use dioxus::prelude::*;
use egg_mode::user::TwitterUser;

use crate::config::Config;
use crate::storage::MediaResolver;

use egg_mode::tweet::Tweet;

use super::helpers::{BottomSpacer, ShowMoreButton};
use super::tweet_component::TweetComponent;

#[derive(Props)]
pub struct TweetListProps<'a> {
    data: &'a [Tweet],
    media: MediaResolver<'a>,
    user: &'a TwitterUser,
    responses: &'a HashMap<u64, Vec<Tweet>>,
    label: String,
    config: &'a Config,
}

pub fn TweetListComponent<'a>(cx: Scope<'a, TweetListProps>) -> Element<'a> {
    let page_size = 100;
    let page = use_state(&cx, || page_size);
    let inverse = use_state(&cx, || false);
    let has_more = cx.props.data.len() > *page.get();
    let iter: Box<dyn Iterator<Item = &Tweet>> = if *inverse.get() {
        Box::new(cx.props.data.iter().rev())
    } else {
        Box::new(cx.props.data.iter())
    };
    let tweets_rendered = iter.take(*page.get()).map(|tweet| {
        let responses = cx.props.responses.get(&tweet.id).as_ref().map(|e| e.len());
        cx.render(rsx!(TweetComponent {
            tweet: tweet,
            media: cx.props.media.clone(),
            user: cx.props.user
            responses: responses,
            config: cx.props.config
        }))
    });

    cx.render(rsx!(div {
        div {
            class: "hstack gap-3",
            h5 {
                "{cx.props.label}"
            }
            button {
                class: "btn btn-outline-info ms-auto",
                r#type: "button",
                onclick: move |_| {
                    page.set(page_size);
                    inverse.set(!(*inverse.get()));
                },
                "\u{21F5}"
            }
        }
        tweets_rendered
        ShowMoreButton {
            visible: has_more,
            onclick: move |_| page.set(page.get() + page_size)
        }
        BottomSpacer {}
    }
    ))
}
