#![allow(non_snake_case)]

use dioxus::fermi::use_atom_state;
use dioxus::prelude::*;
use dioxus_heroicons::solid::Shape;
use dioxus_heroicons::Icon;

use crate::config::Config;
use crate::storage::{List, TweetId, UserId};

use super::primary_column::MainColumn;
use super::secondary_column::SecondaryColumn;
use super::types::{LoadingState, StorageWrapper};

pub const TWATVAULT_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-box2-heart" viewBox="0 0 16 16"><path d="M8 7.982C9.664 6.309 13.825 9.236 8 13 2.175 9.236 6.336 6.31 8 7.982Z"/><path d="M3.75 0a1 1 0 0 0-.8.4L.1 4.2a.5.5 0 0 0-.1.3V15a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1V4.5a.5.5 0 0 0-.1-.3L13.05.4a1 1 0 0 0-.8-.4h-8.5Zm0 1H7.5v3h-6l2.25-3ZM8.5 4V1h3.75l2.25 3h-6ZM15 5v10H1V5h14Z"/></svg>"#;

#[derive(PartialEq, Eq, Clone)]
pub enum Tab {
    Tweets,
    Mentions,
    Likes,
    Followers,
    Follows,
    Lists,
    Search,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Tweets => f.write_str("Tweets"),
            Tab::Mentions => f.write_str("Mentions"),
            Tab::Likes => f.write_str("Likes"),
            Tab::Followers => f.write_str("Followers"),
            Tab::Follows => f.write_str("Follows"),
            Tab::Lists => f.write_str("Lists"),
            Tab::Search => f.write_str("Search"),
        }
    }
}

impl Tab {
    fn icon(&self) -> Shape {
        match self {
            Tab::Tweets => Shape::Home,
            Tab::Mentions => Shape::ChatAlt2,
            Tab::Likes => Shape::Heart,
            Tab::Followers => Shape::Users,
            Tab::Follows => Shape::UserGroup,
            Tab::Lists => Shape::ViewList,
            Tab::Search => Shape::SearchCircle,
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum ColumnState {
    /// Any kind of tweet. will search all data until it is found
    AnyTweet(TweetId),
    /// Responses to a tweet
    Responses(TweetId),
    /// A given profile
    Profile(UserId),
    /// A list
    List(List),
    /// Nothing in the clumn
    None,
}

pub static COLUMN2: Atom<ColumnState> = |_| ColumnState::None;

pub fn Divider(cx: Scope) -> Element {
    cx.render(rsx!(div {
        style: "flex-shrink: 0; width: 1.5rem; height: 100vh; background-color: rgba(0, 0, 0, .1); border: solid rgba(0, 0, 0, .15); border-width: 1px 0; box-shadow: inset 0 .5em 1.5em rgba(0, 0, 0, .1), inset 0 .125em .5em rgba(0, 0, 0, .15)",
        " "
    }))
}

pub fn Remainder(cx: Scope) -> Element {
    cx.render(rsx!(div {
        style: "width: 100vh; height: 100vh; background-color: rgba(0, 0, 0, .1); border: solid rgba(0, 0, 0, .15); border-width: 1px 0; box-shadow: inset 0 .5em 1.5em rgba(0, 0, 0, .1), inset 0 .125em .5em rgba(0, 0, 0, .15)",
        " "
    }))
}

#[inline_props]
pub fn MainComponent(
    cx: Scope,
    config: Config,
    storage: StorageWrapper,
    state: UseState<LoadingState>,
) -> Element {
    let selected = use_state(&cx, || Tab::Tweets);

    let column2 = use_atom_state(&cx, COLUMN2);
    let is_column2 = column2.current().as_ref() != &ColumnState::None;

    cx.render(rsx! {
        main {
            class: "d-flex flex-nowrap",
            div {
                class: "d-flex flex-column bg-dark",
                style: "width: 4.2rem;",
                // span {
                //     class: "p-4",
                //     i {
                //         class: "bi text-light",
                //         dangerous_inner_html: "{TWATVAULT_ICON}"
                //     }
                // }
                NavElement {
                    label: Tab::Tweets,
                    selected: selected.clone()
                }
                NavElement {
                    label: Tab::Mentions
                    selected: selected.clone()
                }
                NavElement {
                    label: Tab::Likes
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
                NavElement {
                    label: Tab::Lists
                    selected: selected.clone()
                }
                NavElement {
                    label: Tab::Search
                    selected: selected.clone()
                }
                div {
                    class: "m-2 p-2 flex-column d-inline-flex align-items-center",
                    style: "cursor: pointer",
                    onclick: move |_| state.set(LoadingState::Loading({
                        let mut cfg = config.clone();
                        cfg.is_sync = true;
                        cfg
                    })),
                    Icon {
                        icon: Shape::LightningBolt,
                        fill: "white",
                        size: 20
                    }
                    span {
                        class: "text-light",
                        style: "font-size: .55rem",
                        "Sync"
                    }
                }
                div {
                    class: "mt-auto d-flex align-items-center text-center",
                     style: "margin-bottom: 20px; gap: 4px; color: white; margin-left: 4px;",
                    a {
                        href: "https://mastodon.social/@terhechte",
                        dangerous_inner_html: r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="white" class="bi bi-mastodon" viewBox="0 0 16 16"><path d="M11.19 12.195c2.016-.24 3.77-1.475 3.99-2.603.348-1.778.32-4.339.32-4.339 0-3.47-2.286-4.488-2.286-4.488C12.062.238 10.083.017 8.027 0h-.05C5.92.017 3.942.238 2.79.765c0 0-2.285 1.017-2.285 4.488l-.002.662c-.004.64-.007 1.35.011 2.091.083 3.394.626 6.74 3.78 7.57 1.454.383 2.703.463 3.709.408 1.823-.1 2.847-.647 2.847-.647l-.06-1.317s-1.303.41-2.767.36c-1.45-.05-2.98-.156-3.215-1.928a3.614 3.614 0 0 1-.033-.496s1.424.346 3.228.428c1.103.05 2.137-.064 3.188-.189zm1.613-2.47H11.13v-4.08c0-.859-.364-1.295-1.091-1.295-.804 0-1.207.517-1.207 1.541v2.233H7.168V5.89c0-1.024-.403-1.541-1.207-1.541-.727 0-1.091.436-1.091 1.296v4.079H3.197V5.522c0-.859.22-1.541.66-2.046.456-.505 1.052-.764 1.793-.764.856 0 1.504.328 1.933.983L8 4.39l.417-.695c.429-.655 1.077-.983 1.934-.983.74 0 1.336.259 1.791.764.442.505.661 1.187.661 2.046v4.203z"/></svg>"#
                    }
                    a {
                        href: "https://twitter.com/terhechte",
                        dangerous_inner_html: r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="white" class="bi bi-twitter" viewBox="0 0 16 16"><path d="M5.026 15c6.038 0 9.341-5.003 9.341-9.334 0-.14 0-.282-.006-.422A6.685 6.685 0 0 0 16 3.542a6.658 6.658 0 0 1-1.889.518 3.301 3.301 0 0 0 1.447-1.817 6.533 6.533 0 0 1-2.087.793A3.286 3.286 0 0 0 7.875 6.03a9.325 9.325 0 0 1-6.767-3.429 3.289 3.289 0 0 0 1.018 4.382A3.323 3.323 0 0 1 .64 6.575v.045a3.288 3.288 0 0 0 2.632 3.218 3.203 3.203 0 0 1-.865.115 3.23 3.23 0 0 1-.614-.057 3.283 3.283 0 0 0 3.067 2.277A6.588 6.588 0 0 1 .78 13.58a6.32 6.32 0 0 1-.78-.045A9.344 9.344 0 0 0 5.026 15z"/></svg>"#
                    }
                    a {
                        href: "https://twitter.com/terhechte",
                        dangerous_inner_html: r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="white" class="bi bi-github" viewBox="0 0 16 16"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>"#
                    }
                }
            }
            Divider()
            MainColumn {
                storage: storage.clone(),
                selected: selected.clone()
                config: config.clone()
            }
            {match is_column2 {
                true => rsx!(Divider()),
                false => rsx!(Remainder())
            }}

            is_column2.then(|| rsx!(
                SecondaryColumn {
                    storage: storage.clone(),
                    selected: column2.clone(),
                    config: config.clone()
                }
            ))

            is_column2.then(|| rsx!(Remainder()))


        }
    })
}

#[inline_props]
fn NavElement(cx: Scope, label: Tab, selected: UseState<Tab>) -> Element {
    let mut classes =
        "p-2 border-bottom border-light flex-column d-inline-flex align-items-center".to_string();
    let icon = label.icon();
    if *selected.current() == *label {
        classes.push_str(" bg-primary");
    }
    rsx!(cx, div {
        class: "{classes}",
        onclick: move |_| selected.set(label.clone()),
        style: "--bs-border-opacity: .3; cursor: pointer;",
        Icon {
            icon: icon,
            fill: "white",
            size: 20
        }
        span {
            class: "text-light",
            style: "font-size: .55rem",
            "{label}"
        }
    })
}
