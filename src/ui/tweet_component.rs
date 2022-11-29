#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf};

use dioxus::fermi::use_atom_state;
use dioxus::prelude::*;
use egg_mode::user::TwitterUser;

use crate::crawler::DownloadInstruction;
use crate::storage::UrlString;

use egg_mode::tweet::Tweet;

use super::main_component::{ColumnState, COLUMN2};
use super::user_component::AuthorImageComponent;

#[derive(Props)]
pub struct TweetProps<'a> {
    tweet: &'a Tweet,
    media: &'a HashMap<UrlString, PathBuf>,
    user: &'a TwitterUser,
    responses: Option<Option<usize>>,
}

pub fn TweetComponent<'a>(cx: Scope<'a, TweetProps>) -> Element<'a> {
    let tweet = cx.props.tweet;
    let date = tweet.created_at.format("%d/%m/%Y %H:%M").to_string();

    let text = formatted_tweet(tweet);

    let media = crate::helpers::media_in_tweet(tweet);

    let image = media
        .as_ref()
        .and_then(|media| {
            media.iter().find_map(|item| match item {
                DownloadInstruction::Image(url) => Some(url.clone()),
                _ => None,
            })
        })
        .map(|entry| {
            cx.props
                .media
                .get(&entry)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| entry.clone())
        })
        .map(|entry| {
            rsx!(img {
                src: "{entry}",
                class: "card-img-bottom img-thumbnail"
            })
        })
        .unwrap_or_else(|| rsx!(div {}));

    let video = media
        .and_then(|media| {
            media.iter().find_map(|item| match item {
                DownloadInstruction::Movie(_, url) => Some(url.clone()),
                _ => None,
            })
        })
        .map(|entry| {
            cx.props
                .media
                .get(&entry)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| entry.clone())
        })
        .map(|entry| {
            rsx!( div {
                class: "ratio ratio-16x9",
                video {
                    controls: "true",
                    source {
                        src: "{entry}"
                    }
            }
            })
        })
        .unwrap_or_else(|| rsx!(div {}));

    let user = tweet.user.as_deref().unwrap_or(cx.props.user);

    let user_image = tweet
        .user
        .as_ref()
        .map(|user| {
            rsx!(AuthorImageComponent {
                profile: user,
                media: cx.props.media
            })
        })
        .unwrap_or_else(|| rsx!(div {}));

    let column2 = use_atom_state(&cx, COLUMN2);

    let tweet_info = rsx!(
        div {
            class: "card-title",
            onclick: move |_| column2.set(ColumnState::Profile(user.id)),
            strong {
                class: "text-dark",
                "{user.name}"
            }
            " "
            "@{user.screen_name}"
            " "
            span {
                class: "text-muted",
                "{date}"
            }
        }
    );

    let tweet_responses = cx.props.responses.flatten().map(|e| {
        rsx!(
            span {
                class: "text-primary",
                onclick: move |_| column2.set(ColumnState::Responses(tweet.id)),
                "{e} Responses"
            }
        )
    });

    let tweet_actions = rsx!(div {
        span {
            class: "text-success",
            "{tweet.favorite_count} Likes"
        }
        " "
        span {
            class: "text-success",
            "{tweet.retweet_count} Retweets"
        }
        " "
        tweet_responses
        " "
        a {
            class: "btn btn-info btn-sm",
            href: "https://twitter.com/{user.screen_name}/status/{tweet.id}",
            "Open on Twitter"
        }
    });

    let quoted = tweet
        .quoted_status
        .as_ref()
        .map(|quoted| {
            rsx!(div {
                TweetComponent {
                    tweet: quoted,
                    media: cx.props.media,
                    user: cx.props.user
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
                user_image
            }
            div {
                class: "col-11 g-0",
                div {
                    class: "card-body",
                    tweet_info
                    p {
                        class: "card-text",
                        dangerous_inner_html: "{text}"
                    }
                    tweet_actions
                    quoted
                    video
                    image
                }
            }
        }
    }))
}

fn formatted_tweet(tweet: &Tweet) -> String {
    let mut output = String::new();
    let mut additions = Vec::new();
    additions.extend(
        tweet
            .entities
            .hashtags
            .iter()
            .map(|tag| (tag.range, "http://test.com".to_string())),
    );
    additions.extend(tweet.entities.urls.iter().map(|url| {
        (
            url.range,
            url.expanded_url
                .as_ref()
                .unwrap_or(&url.display_url)
                .clone(),
        )
    }));

    additions.extend(
        tweet
            .entities
            .user_mentions
            .iter()
            .map(|mention| (mention.range, format!("{}", mention.id))),
    );

    if let Some(media) = tweet.entities.media.as_ref() {
        additions.extend(
            media
                .iter()
                .map(|media| (media.range, media.expanded_url.clone())),
        );
    }

    additions.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

    let t = &tweet.text;

    let mut current = 0;
    for (range, link) in additions {
        // Get the part from last to beginning
        output.push_str(&t[current..range.0]);
        output.push_str(&format!("<a href='{link}'>"));
        output.push_str(&t[range.0..range.1]);
        output.push_str("</a>");
        current = range.1;
    }
    output.push_str(&t[current..t.len()]);

    output
}
