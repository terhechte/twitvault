// ANCHOR: all
#![allow(non_snake_case)]
use std::{collections::HashMap, ops::Deref, path::PathBuf, rc::Rc};

use dioxus::prelude::*;
use egg_mode::account::UserProfile;
use egg_mode::user::TwitterUser;
use tracing::info;

use crate::config::Config;
use crate::crawler::DownloadInstruction;
use crate::storage::{Data, Storage, UrlString};
use egg_mode::tweet::Tweet;

#[derive(Clone)]
enum LoadingState {
    Setup(String),
    Loading(Vec<String>),
    Loaded(StorageWrapper),
}

#[derive(Clone)]
struct StorageWrapper(Rc<Storage>);

impl StorageWrapper {
    fn data(&self) -> &Data {
        self.0.data()
    }
}

impl PartialEq for StorageWrapper {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl Eq for StorageWrapper {}

impl Default for LoadingState {
    fn default() -> Self {
        // TEMPORARY
        //let data = Storage::open("archive_terhechte").unwrap();
        let s = Config::archive_path();
        let data = Storage::open(s).unwrap();
        LoadingState::Loaded(StorageWrapper(Rc::new(data)))
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Tab {
    Tweets,
    Mentions,
    Followers,
    Follows,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Tweets => f.write_str("Tweets"),
            Tab::Mentions => f.write_str("Mentions"),
            Tab::Followers => f.write_str("Followers"),
            Tab::Follows => f.write_str("Follows"),
        }
    }
}

pub fn run_ui() {
    dioxus_desktop::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.use_hook(|_| {
        cx.provide_context(LoadingState::default());
    });

    let data = cx.use_hook(|_| cx.consume_context::<LoadingState>());

    let view = match data {
        Some(LoadingState::Loading(_)) => cx.render(rsx! {
            LoadingComponent {

            }
        }),
        Some(LoadingState::Loaded(store)) => cx.render(rsx! {
            LoadedComponent {
                storage: store.clone()
            }
        }),
        Some(LoadingState::Setup(url)) => cx.render(rsx! {
            SetupComponent {
                url: url.clone()
            }
        }),
        None => todo!(),
    };

    rsx!(cx, div {
        link {
            href: "https://cdn.jsdelivr.net/npm/bootstrap@5.2.3/dist/css/bootstrap.min.css",
            rel: "stylesheet",
            crossorigin: "anonymous"
        },
        link {
            href: "assets/style.css",
            rel: "stylesheet",
            crossorigin: "anonymous"
        },
        view
    })
}

#[inline_props]
fn SetupComponent(cx: Scope, url: String) -> Element {
    cx.render(rsx! {
        div {
            a {
                href: "{ url }"
            }
        }
    })
}

#[inline_props]
fn LoadedComponent(cx: Scope, storage: StorageWrapper) -> Element {
    let selected = use_state(&cx, || Tab::Tweets);

    let data = match *selected.current() {
        Tab::Tweets => &storage.data().tweets[0..50],
        Tab::Mentions => &storage.data().mentions[0..50],
        _ => panic!(),
    };
    let label = (*selected.current()).to_string();

    cx.render(rsx! {
        main {
            class: "d-flex flex-nowrap",
            div {
                class: "d-flex flex-column flex-shrink-0 bg-light",
                style: "width: 6.5rem;",
                ul {
                    class: "nav nav-pills nav-flush flex-column mb-auto text-center",
                    NavElement {
                        label: Tab::Tweets,
                        selected: selected.clone()
                    }
                    NavElement {
                        label: Tab::Mentions
                        selected: selected.clone()
                    }
                }
            }
            Divider()
            div {
                class: "d-flex flex-column flex-shrink-0 bg-light",
                style: "width: 35rem; overflow: scroll; padding: 12px; height: 100vh;",
                TweetListComponent {
                    data: data,
                    media: &storage.data().media,
                    label: label,
                    user: &storage.data().profile,
                    responses: &storage.data().responses
                }
            }
            Divider()
        }
    })
}

fn Divider(cx: Scope) -> Element {
    cx.render(rsx!(div {
        style: "flex-shrink: 0; width: 1.5rem; height: 100vh; background-color: rgba(0, 0, 0, .1); border: solid rgba(0, 0, 0, .15); border-width: 1px 0; box-shadow: inset 0 .5em 1.5em rgba(0, 0, 0, .1), inset 0 .125em .5em rgba(0, 0, 0, .15)",
        " "
    }))
}

#[inline_props]
fn NavElement(cx: Scope, label: Tab, selected: UseState<Tab>) -> Element {
    let mut classes = "nav-link py-3 border-bottom rounded-0".to_string();
    if *selected.current() == *label {
        classes.push_str(" active");
    }
    rsx!(cx, li {
        class: "nav-item",
        a {
            class: "{classes}",
            onclick: move |_| selected.set(label.clone()),
            href: "#",
            "{label}"
        }
    })
}

#[inline_props]
fn LoadingComponent(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            h1 { "Loading" }
        }
    })
}

#[inline_props]
fn ErrorComponent(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            h1 { "Error" }
        }
    })
}

#[derive(Props)]
struct TweetListProps<'a> {
    data: &'a [Tweet],
    media: &'a HashMap<UrlString, PathBuf>,
    user: &'a TwitterUser,
    responses: &'a HashMap<u64, Vec<Tweet>>,
    label: String,
}

fn TweetListComponent<'a>(cx: Scope<'a, TweetListProps>) -> Element<'a> {
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

#[derive(Props)]
struct TweetProps<'a> {
    tweet: &'a Tweet,
    media: &'a HashMap<UrlString, PathBuf>,
    user: &'a TwitterUser,
    responses: Option<Option<usize>>,
}

fn TweetComponent<'a>(cx: Scope<'a, TweetProps>) -> Element<'a> {
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
        .and_then(|entry| cx.props.media.get(&entry).map(|path| path.display()))
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
        .and_then(|entry| cx.props.media.get(&entry).map(|path| path.display()))
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
            rsx!(AuthorComponent {
                profile: user,
                media: cx.props.media
                user: cx.props.user
            })
        })
        .unwrap_or_else(|| rsx!(div {}));

    let tweet_info = rsx!(
        div {
            class: "card-title",
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

#[derive(Props)]
struct AuthorProps<'a> {
    profile: &'a TwitterUser,
    media: &'a HashMap<UrlString, PathBuf>,
    user: &'a TwitterUser,
}

fn AuthorComponent<'a>(cx: Scope<'a, AuthorProps>) -> Element<'a> {
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
