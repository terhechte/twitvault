#![allow(non_snake_case)]

use dioxus::fermi::use_atom_state;
use dioxus::prelude::*;
use egg_mode::user::TwitterUser;
use tracing::info;

use crate::config::Config;
use crate::crawler::DownloadInstruction;
use crate::helpers::delete_tweet;
use crate::storage::MediaResolver;

use egg_mode::tweet::Tweet;

use super::main_component::{ColumnState, COLUMN2};
use super::user_component::AuthorImageComponent;

#[derive(Props)]
pub struct TweetProps<'a> {
    tweet: &'a Tweet,
    media: MediaResolver<'a>,
    user: &'a TwitterUser,
    responses: Option<Option<usize>>,
    config: &'a Config,
}

pub fn TweetComponent<'a>(cx: Scope<'a, TweetProps>) -> Element<'a> {
    let tweet = cx.props.tweet;

    let user = tweet.user.as_deref().unwrap_or(cx.props.user);

    let column2 = use_atom_state(&cx, COLUMN2);

    let date = tweet.created_at.format("%d/%m/%y %H:%M").to_string();

    let pure_text = &tweet.text;

    let text = formatted_tweet(tweet);

    let media = crate::helpers::media_in_tweet(tweet);

    let modal_id = format!("modal-{}", tweet.id);

    // we can only delete our own tweets
    let can_delete = cx.props.user.id == user.id;

    // The deletion action
    let cloned_config = cx.props.config.clone();
    let deletion_tweet: &UseState<Option<u64>> = use_state(&cx, || None);
    let deletion_future = use_future(&cx, deletion_tweet, |oid| async move {
        let Some(id) = oid.get() else {
            return None
        };
        Some(delete_tweet(*id, &cloned_config).await)
    });

    let action_dropdown = rsx! {
        div {
            class: "dropdown",
            button {
                "aria-expanded": "false",
                "data-bs-toggle": "dropdown",
                style: "font-weight: bold; --bs-btn-padding-y: .1rem; --bs-btn-padding-x: .3rem; --bs-btn-font-size: .95rem;",
                class: "btn btn-outline-secondary drowdopwn-toggle",
                r#type: "button",
                " \u{2807}"
            }
            ul {
                class: "dropdown-menu",
                li {
                    a {
                        class: "dropdown-item fs-6",
                        href: "https://twitter.com/{user.screen_name}/status/{tweet.id}",
                        "Open on Twitter"
                    }
                }
                li {
                    a {
                        class: "dropdown-item",
                        href: "https://twitter.com/{user.screen_name}",
                        "Open Author on Twitter"
                    }
                }
                li {
                    a {
                        class: "dropdown-item",
                        onclick: move |_| column2.set(ColumnState::Profile(user.id)),
                        "Go to Author"
                    }
                }
                li {
                    hr {
                        class: "dropdown-divider"
                    }
                }
                { can_delete.then(|| rsx!(li {
                    button {
                        class: "dropdown-item btn btn-danger text-danger",
                        r#type: "button",
                        "data-bs-toggle": "modal",
                        "data-bs-target": "#{modal_id}",
                        "Delete on Twitter"
                    }
                })) }
            }
        }
    };
    let modal_id = format!("modal-{}", tweet.id);

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
                .resolve(&entry)
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
                .resolve(&entry)
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

    let user_image = tweet
        .user
        .as_ref()
        .map(|user| {
            rsx!(AuthorImageComponent {
                profile: user,
                media: cx.props.media.clone()
            })
        })
        .unwrap_or_else(|| rsx!(div {}));

    let tweet_info = rsx!(
        div {
            class: "card-title d-flex flex-row justify-content-between align-items-center",
            style: "font-size: 13px; gap: 5px;",
            strong {
                class: "text-dark",
                onclick: move |_| column2.set(ColumnState::Profile(user.id)),
                "{user.name}"
            }
            span {
                onclick: move |_| column2.set(ColumnState::Profile(user.id)),
                "@{user.screen_name}"
            }
            span {
                class: "text-muted me-auto",
                style: "font-size: 12px",
                "{date}"
            }
            action_dropdown
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
        style: "margin-bottom: 8px;",
        small {
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
            }
    });

    let quoted = tweet
        .quoted_status
        .as_ref()
        .map(|quoted| {
            rsx!(div {
                TweetComponent {
                    tweet: quoted,
                    media: cx.props.media.clone(),
                    user: cx.props.user
                    responses: None
                    config: cx.props.config
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
            div {
                class: "modal",
                "data-bs-backdrop": "false",
                id: "{modal_id}",
                div {
                    class: "modal-dialog",
                    div {
                        class: "modal-content",
                        div {
                            class: "modal-body",
                            "Do you really want to delete this tweet?"
                            div {
                                class: "hstack gap-2",
                                div {
                                    class: "vr"
                                }
                                p {
                                    small {
                                        em {
                                            "{pure_text}"
                                        }
                                    }
                                }
                            }
                            div {
                                class: "alert alert-warning",
                                "The tweet will be deleted on Twitter but will still be available in this archive."
                            }
                        }
                        div {
                            class: "modal-footer",
                            button {
                                class: "btn btn-secondary",
                                "data-bs-dismiss": "modal",
                                r#type: "button",
                                "Don't delete"
                            }
                            button {
                                class: "btn btn-danger",
                                "data-bs-dismiss": "modal",
                                r#type: "button",
                                onclick: move |_| {
                                    deletion_tweet.set(Some(tweet.id));
                                    deletion_future.restart();
                                },
                                "Yes, delete please"
                            }
                        }
                    }
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
