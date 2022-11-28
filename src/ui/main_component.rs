#![allow(non_snake_case)]

use dioxus::fermi::use_atom_state;
use dioxus::prelude::*;

use crate::storage::{TweetId, UserId};

use super::primary_column::MainColumn;
use super::secondary_column::SecondaryColumn;
use super::types::StorageWrapper;

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

pub static COLUMN2: Atom<ColumnState> = |_| ColumnState::None;

pub fn Divider(cx: Scope) -> Element {
    cx.render(rsx!(div {
        style: "flex-shrink: 0; width: 1.5rem; height: 100vh; background-color: rgba(0, 0, 0, .1); border: solid rgba(0, 0, 0, .15); border-width: 1px 0; box-shadow: inset 0 .5em 1.5em rgba(0, 0, 0, .1), inset 0 .125em .5em rgba(0, 0, 0, .15)",
        " "
    }))
}

#[inline_props]
pub fn MainComponent(cx: Scope, storage: StorageWrapper) -> Element {
    let selected = use_state(&cx, || Tab::Tweets);

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
