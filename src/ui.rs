#![allow(non_snake_case)]
use std::{collections::HashMap, ops::Deref, path::PathBuf, rc::Rc};

use dioxus::core::to_owned;
use dioxus::fermi::{use_atom_root, use_atom_state, AtomState};
use dioxus::prelude::*;
use egg_mode::account::UserProfile;
use egg_mode::user::TwitterUser;
use eyre::Report;
use futures::StreamExt;
use tokio::sync::mpsc::channel;
use tracing::{info, warn};

use crate::config::Config;
use crate::crawler::DownloadInstruction;
use crate::storage::{Data, List, Storage, TweetId, UrlString, UserId};
use crate::types::Message;
use egg_mode::tweet::Tweet;

#[derive(Clone)]
enum LoadingState {
    Setup(String),
    Loading(Vec<String>),
    Loaded(StorageWrapper),
    Error(String),
}

#[derive(Clone)]
struct StorageWrapper {
    data: Rc<Storage>,
    empty_tweets: Vec<Tweet>,
}

impl StorageWrapper {
    fn new(storage: Storage) -> Self {
        Self {
            data: Rc::new(storage),
            empty_tweets: Vec::new(),
        }
    }

    fn data(&self) -> &Data {
        self.data.data()
    }
}

impl PartialEq for StorageWrapper {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for StorageWrapper {}

impl Default for LoadingState {
    fn default() -> Self {
        // TEMPORARY
        //let data = Storage::open("archive_terhechte").unwrap();

        // let s = Config::archive_path();
        // let data = Storage::open(s).unwrap();
        // LoadingState::Loaded(StorageWrapper::new(data))
        LoadingState::Setup(String::new())
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

#[derive(PartialEq, Eq, Clone)]
pub enum ColumnState {
    /// Responses to a tweet
    Responses(TweetId),
    /// A given profile
    Profile(UserId),
    /// Nothing in the clumn
    None,
}

static COLUMN2: Atom<ColumnState> = |_| ColumnState::None;

pub fn run_ui() {
    dioxus::desktop::launch(App);
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
        Some(LoadingState::Error(e)) => cx.render(rsx! {
            div {
                "{e}"
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
        view
    })
}

#[inline_props]
fn SetupComponent(cx: Scope, url: String) -> Element {
    let loading_state = use_state(&cx, || Message::Loading(String::new()));

    let crawl = move |config: Config| {
        let (sender, mut receiver) = channel(256);
        cx.spawn(async move {
            let path = Config::archive_path();
            if let Err(e) = crate::crawler::crawl_new_storage(config, &path, sender).await {
                warn!("Error {e:?}");
            }
        });
        // cx.spawn(async move {
        use_future(&cx, (), move |_| {
            let mut loading_state = loading_state.clone();
            async move {
                while let Some(msg) = receiver.recv().await {
                    let finished = match &msg {
                        Message::Finished(_) => true,
                        _ => false,
                    };
                    loading_state.set(msg);
                    if finished {
                        break;
                    }
                }
            }
        });
        // });
    };

    // FIXME: Setup-config flow
    let config = Config::open().unwrap();

    cx.render(rsx! {
        div {
            div {
                "{loading_state}"
            }
            button {
                onclick: move |_| crawl(config.clone())
            }
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
fn LoadedComponent(cx: Scope, storage: StorageWrapper) -> Element {
    let selected = use_state(&cx, || Tab::Tweets); //use_atom_state(&cx, COLUMN1);

    let column2 = use_atom_state(&cx, COLUMN2);
    let is_column2 = column2.current().as_ref() != &ColumnState::None;

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
                    NavElement {
                        label: Tab::Follows
                        selected: selected.clone()
                    }
                    NavElement {
                        label: Tab::Followers
                        selected: selected.clone()
                    }
                }
            }
            Divider()
            MainColumn {
                storage: storage.clone(),
                selected: selected.clone()
            }
            is_column2.then(|| rsx!(div {
                SecondaryColumn {
                    storage: storage.clone(),
                    selected: column2.clone()
                }
                Divider()
            }
            ))
        }
    })
}

#[inline_props]
fn MainColumn(cx: Scope, storage: StorageWrapper, selected: UseState<Tab>) -> Element {
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
                        data: &storage.data().tweets[0..50],
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
                        data: &storage.data().mentions[0..50],
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

#[inline_props]
fn SecondaryColumn(
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
            class: "btn btn-secondary",
            onclick: move |_| selected.set(ColumnState::None),
            "Close"
        }
        column
    }})
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
            rsx!(AuthorImageComponent {
                profile: user,
                media: cx.props.media
                user: cx.props.user
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

#[derive(Props)]
struct AuthorImageProps<'a> {
    profile: &'a TwitterUser,
    media: &'a HashMap<UrlString, PathBuf>,
    user: &'a TwitterUser,
}

fn AuthorImageComponent<'a>(cx: Scope<'a, AuthorImageProps>) -> Element<'a> {
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

#[derive(Props)]
struct AuthorListProps<'a> {
    data: &'a [u64],
    media: &'a HashMap<UrlString, PathBuf>,
    profiles: &'a HashMap<u64, TwitterUser>,
    label: String,
}

fn AuthorListComponent<'a>(cx: Scope<'a, AuthorListProps>) -> Element<'a> {
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

#[derive(Props)]
struct AuthorProps<'a> {
    profile: &'a TwitterUser,
    media: &'a HashMap<UrlString, PathBuf>,
}

fn AuthorComponent<'a>(cx: Scope<'a, AuthorProps>) -> Element<'a> {
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
                    user: author
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
