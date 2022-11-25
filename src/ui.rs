// ANCHOR: all
#![allow(non_snake_case)]
use std::{collections::HashMap, ops::Deref, path::PathBuf, rc::Rc};

use dioxus::prelude::*;

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
        let data = Storage::open("archive_terhechte").unwrap();
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
        Tab::Tweets => &storage.data().tweets,
        Tab::Mentions => &storage.data().mentions,
        _ => panic!(),
    };

    cx.render(rsx! {
        main {
            class: "d-flex flex-nowrap",
            div {
                class: "d-flex flex-column flex-shrink-0 bg-light",
                style: "width: 6.5rem",
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
            TweetListComponent {
                data: data,
                media: &storage.data().media
            }
        }
    })
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
}

fn TweetListComponent<'a>(cx: Scope<'a, TweetListProps>) -> Element<'a> {
    let tweets_rendered = cx.props.data.iter().map(|tweet| {
        cx.render(rsx!(TweetComponent {
            tweet: tweet,
            media: cx.props.media
        }))
    });

    cx.render(rsx!(div {
        h1 { "Hello: " },
        tweets_rendered
    }
    ))
}

#[derive(Props)]
struct TweetProps<'a> {
    tweet: &'a Tweet,
    media: &'a HashMap<UrlString, PathBuf>,
}

fn TweetComponent<'a>(cx: Scope<'a, TweetProps>) -> Element<'a> {
    let image = if let Some(n) = cx
        .props
        .tweet
        .extended_entities
        .as_ref()
        .and_then(|e| e.media.first().cloned())
        .and_then(|e| cx.props.media.get(&e.expanded_url))
    {
        n.display().to_string()
    } else {
        "".to_string()
    };

    let date = cx
        .props
        .tweet
        .created_at
        .format("%d/%m/%Y %H:%M")
        .to_string();

    let text = &cx.props.tweet.text;

    cx.render(rsx!(div {
        class: "card",
        style: "margin: 8px",
        div {
            img {
                src: "{image}",
                class: "card-img-top"
            }
        }
        div {
            class: "card-body",
            h5 {
                class: "card-title",
                "{date}"
            }
            p {
                class: "card-text",
                "{text}"
            }
        }
    }))
}
