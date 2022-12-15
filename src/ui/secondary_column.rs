#![allow(non_snake_case)]

use std::borrow::Cow;

use dioxus::fermi::{use_atom_state, AtomState};
use dioxus::prelude::*;

use crate::config::Config;
use crate::ui::user_list::AuthorListComponent;

use super::helpers::Box;
use super::main_component::{ColumnState, COLUMN2};
use super::tweet_component::TweetComponent;
use super::tweet_list::TweetListComponent;
use super::types::StorageWrapper;
use super::user_component::AuthorComponent;

#[inline_props]
pub fn SecondaryColumn(
    cx: Scope,
    storage: StorageWrapper,
    selected: AtomState<ColumnState>,
    config: Config,
) -> Element {
    let column2 = use_atom_state(&cx, COLUMN2);

    if column2.current().as_ref() == &ColumnState::None {
        return cx.render(rsx! { div { }});
    }

    let column_class = "d-flex flex-column flex-shrink-0 bg-light";
    let column_style =
        "flex-basis: 28rem; width: 28rem; overflow: scroll; padding: 8; height: 100vh;";

    let column = cx.render(rsx!(div {
        {if let ColumnState::Responses(id) = column2.current().as_ref() {
            let label = "Responses".to_string();
            rsx!{
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    TweetListComponent {
                        data: storage.data().responses.get(id).unwrap_or(&storage.empty_tweets),
                        media: storage.resolver(),
                        label: label,
                        user: &storage.data().profile,
                        responses: &storage.data().responses,
                        config: &config
                    }
                }
            }
        } else {rsx!{ div {} }}}

        {if let ColumnState::List(ref list) = column2.current().as_ref() {
            let label = format!("List: {}", list.name);
            rsx!{
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    AuthorListComponent {
                        data: Cow::Owned(list.members.clone()),
                        media: storage.resolver(),
                        profiles: &storage.data().profiles,
                        label: label
                        config: &config
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
                            media: storage.resolver(),
                            config: &config
                        }
                    }
                }
            } else {
                rsx! { div {
                    class: "p-3",
                    Box {
                        title: "Unknown Profile"
                        p {
                            class: "m-4",
                            "Profile for user with id  "
                            strong {
                                "{id} "
                            }
                            "not found"
                        }
                    }
                }}
            }
        } else {rsx!{ div {} }}}

        {if let ColumnState::AnyTweet(id) = column2.current().as_ref() {
            if let Some(tweet) = storage.data().any_tweet(*id) {
                rsx!{
                    div {
                        class: "{column_class}",
                        style: "{column_style}",
                        TweetComponent {
                            tweet: tweet,
                            media: storage.resolver(),
                            user: &storage.data().profile,
                            responses: None,
                            config: &config
                        }
                    }
                }
            } else {
                rsx! { div {
                    class: "p-3",
                    Box {
                        title: "Unknown Tweet"
                        p {
                            class: "m-4",
                            "Tweet for id "
                            strong {
                                "{id} "
                            }
                            "not found"
                        }
                    }
                }}
            }
        } else {rsx!{ div {} }}}

    }));

    cx.render(rsx! {div {
        class: "vstack gap-2",
        button {
            r#type: "button",
            class: "btn btn-secondary m-2",
            onclick: move |_| selected.set(ColumnState::None),
            "Close"
        }
        column
    }})
}
