#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf};

use dioxus::fermi::use_atom_state;
use dioxus::prelude::*;
use egg_mode::user::TwitterUser;

use crate::storage::UrlString;

use super::main_component::{ColumnState, COLUMN2};
use super::tweet_component::TweetComponent;

#[derive(Props)]
pub struct AuthorProps<'a> {
    profile: &'a TwitterUser,
    media: &'a HashMap<UrlString, PathBuf>,
}

pub fn AuthorComponent<'a>(cx: Scope<'a, AuthorProps>) -> Element<'a> {
    let author = cx.props.profile;
    let date = author.created_at.format("%d/%m/%Y %H:%M").to_string();
    let description = author.description.as_ref().cloned().unwrap_or_default();
    let followers = author.followers_count;
    let follows = author.friends_count;
    let name = author.name.clone();
    let screen_name = author.screen_name.clone();
    let tweets = author.statuses_count;
    let info = rsx!(div {
        strong {
            "{name}"
        }
        ", "
        span {
            class: "text-muted",
            "{screen_name}"
        }
        " "
        span {
            class: "text-muted",
            "Joined {date}"
        }
    });
    let numbers = rsx!(div {
        span {
            class: "text-success",
            "Followers {followers}"
        }
        ", "
        span {
            class: "text-success",
            "Follows {follows}"
        }
        ", "
        span {
            class: "text-success",
            "Tweets {tweets}"
        }
    });

    let url_button = author
        .url
        .as_ref()
        .and_then(|s| url::Url::parse(s).ok().map(|u| (u, s)))
        .and_then(|(url, s)| url.domain().map(|e| (e.to_string(), s)))
        .map(|(domain, url)| {
            rsx!(a {
                class: "btn btn-primary",
                href: "{url}",
                "Link: {domain}"
            })
        });
    let twitter_button = rsx!(a {
        class: "btn btn-primary",
        href: "https://twitter.com/{author.screen_name}",
        "On Twitter"
    });
    let quoted = author
        .status
        .as_ref()
        .map(|quoted| {
            rsx!(div {
                TweetComponent {
                    tweet: quoted,
                    media: cx.props.media,
                    user: cx.props.profile
                    responses: None
                }
            })
        })
        .unwrap_or_else(|| rsx!(div {}));
    cx.render(rsx!(div {
        class: "card",
        style: "margin: 12px",
        div {
            class: "row g-0",
            div {
                class: "col-1 g-0",
                AuthorImageComponent {
                    profile: author,
                    media: cx.props.media
                }
            }
            div {
                class: "col-11 g-0",
                div {
                    class: "card-body",
                    info
                    numbers
                    p {
                        class: "card-text",
                        "{description}"
                    }
                    url_button
                    " "
                    twitter_button
                    quoted
                }
            }
        }
    }))
}

#[derive(Props)]
pub struct AuthorImageProps<'a> {
    profile: &'a TwitterUser,
    media: &'a HashMap<UrlString, PathBuf>,
}

pub fn AuthorImageComponent<'a>(cx: Scope<'a, AuthorImageProps>) -> Element<'a> {
    let column2 = use_atom_state(&cx, COLUMN2);
    let url = &cx.props.profile.profile_image_url_https;
    let node = cx
        .props
        .media
        .get(url)
        .map(|entry| entry.display())
        .map(|entry| {
            rsx!(
                div {
                    style: "margin: 0.6rem; margin-top: 0.8rem;",
                    onclick: move |_| column2.set(ColumnState::Profile(cx.props.profile.id)),
                    img {
                        style: "border-radius: 50%; width: 2rem; height: 2rem;",
                        src: "{entry}",
                    }
                }
            )
        })
        .unwrap_or_else(|| rsx!(div {}));

    cx.render(node)
}
