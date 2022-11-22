// ANCHOR: all
#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf, rc::Rc};

use dioxus::prelude::*;

use crate::storage::{Data, Storage, UrlString};
use egg_mode::tweet::Tweet;

#[derive(Clone)]
enum UiState {
    Setup(String),
    Loading(Vec<String>),
    Loaded(Rc<Storage>),
}

#[derive(Clone)]
struct ViewStore {
    state: UiState,
}

impl Default for ViewStore {
    fn default() -> Self {
        // TEMPORARY
        let data = Storage::open("archive").unwrap();
        Self {
            state: UiState::Loaded(Rc::new(data)),
        }
    }
}

pub fn run_ui() {
    dioxus_desktop::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.use_hook(|_| {
        cx.provide_context(ViewStore::default());
    });

    let data = cx.use_hook(|_| cx.consume_context::<ViewStore>().map(|e| e.state));

    let view = match data {
        Some(UiState::Loading(_)) => cx.render(rsx! {
            LoadingComponent {

            }
        }),
        Some(UiState::Loaded(store)) => cx.render(rsx! {
            TweetListComponent {
                data: &store.data().tweets,
                media: &store.data().media
            }
        }),
        Some(UiState::Setup(url)) => cx.render(rsx! {
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
        main {
            class: "d-flex flex-nowrap",
            div {
                class: "d-flex flex-column flex-shrink-0 bg-light",
                style: "width: 6.5rem",
                ul {
                    class: "nav nav-pills nav-flush flex-column mb-auto text-center",
                    NavElement {
                        label: "Tweets".to_string(),
                        selected: true
                    }
                    NavElement {
                        label: "Tweets".to_string(),
                        selected: false
                    }
                }
            }
            view
        }
    })
}

#[inline_props]
fn NavElement(cx: Scope, label: String, selected: bool) -> Element {
    let mut classes = "nav-link py-3 border-bottom rounded-0".to_string();
    if *selected {
        classes.push_str(" active");
    }
    rsx!(cx, li {
        class: "nav-item",
        a {
            class: "{classes}",
            href: "#",
            "{label}"
        }
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
