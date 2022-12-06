#![allow(non_snake_case)]

use std::borrow::Cow;

use dioxus::prelude::*;

use super::list_list::ListListComponent;
use super::main_component::Tab;
use super::search_list::SearchComponent;
use super::tweet_list::TweetListComponent;
use super::types::StorageWrapper;
use super::user_list::AuthorListComponent;

#[inline_props]
pub fn MainColumn(cx: Scope, storage: StorageWrapper, selected: UseState<Tab>) -> Element {
    let current = (*selected.current()).clone();
    let label = current.to_string();

    let column_class = "d-flex flex-column flex-shrink-0 bg-light";
    let column_style = "width: 30rem; overflow: scroll; padding: 8px; height: 100vh;";

    cx.render(rsx!(div {
        {if current == Tab::Tweets {
            let label = label.clone();
            rsx!{
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    TweetListComponent {
                        data: &storage.data().tweets,
                        media: storage.resolver(),
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
                        data: &storage.data().mentions,
                        media: storage.resolver(),
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
                        data: Cow::Borrowed(&storage.data().follows),
                        media: storage.resolver(),
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
                        data: Cow::Borrowed(&storage.data().followers),
                        media: storage.resolver(),
                        profiles: &storage.data().profiles,
                        label: label.clone(),
                    }
                }
            }
        } else {rsx!{ div {}}}}
        {if current == Tab::Lists {
            rsx! {
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    ListListComponent {
                        lists: &storage.data().lists
                    }
                }
            }
        } else {rsx!{ div {}}}}
        {if current == Tab::Search {
            rsx! {
                div {
                    class: "{column_class}",
                    style: "{column_style}",
                    SearchComponent {
                        storage: storage.clone()
                    }
                }
            }
        } else {rsx!{ div {}}}}
    }))
}
