#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::config::Config;

use super::types::LoadingState;

use super::helpers::{Box, Checkbox, NextButton};

#[inline_props]
pub fn SetupComponent(cx: Scope, config: Config, loading_state: UseState<LoadingState>) -> Element {
    let params = use_state(&cx, move || config.crawl_options().clone());
    cx.render(rsx! { Box {
        title: "Setup Config"
        div {
            class: "vstack gap-2",
            div {
                class: "list-group mx-0 w-auto",
                Checkbox {
                    name: "Your Feed",
                    label: "All your tweets, up to 3200",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.tweets = !o.tweets)),
                    checked: params.get().tweets,
                    disabled: false,
                }
                Checkbox {
                    name: "Responses",
                    label: "All the responses to your tweets",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.tweet_responses = !o.tweet_responses)),
                    checked: params.get().tweet_responses,
                    disabled: !params.get().tweets,
                }
                Checkbox {
                    name: "Mentions",
                    label: "The tweets mentioning you",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.mentions = !o.mentions)),
                    checked: params.get().mentions,
                    disabled: false
                }
                Checkbox {
                    name: "User Profiles",
                    label: "From Responses and Mentions",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.tweet_profiles = !o.tweet_profiles)),
                    checked: params.get().tweet_profiles,
                    disabled: false
                }
                Checkbox {
                    name: "Followers",
                    label: "All your followers with profiles",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.followers = !o.followers)),
                    checked: params.get().followers,
                    disabled: false
                }
                Checkbox {
                    name: "Follows",
                    label: "All your follows with profiles",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.follows = !o.follows)),
                    checked: params.get().follows,
                    disabled: false
                }
                Checkbox {
                    name: "Lists",
                    label: "Lists and the profiles of the members",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.lists = !o.lists)),
                    checked: params.get().lists,
                    disabled: false
                }
                Checkbox {
                    name: "Media",
                    label: "Tweet media (images, videos) and Profile images",
                    onclick: move |_| params.modify(|e| e.changed(|o| o.media = !o.media)),
                    checked: params.get().media,
                    disabled: false
                }
            }
            params.tweet_responses.then(|| {
                rsx!(div {
                    class: "alert alert-primary",
                    "Tweet responses take a long time to load. Up to 3 hours per 1000 tweets"
                })
            })
            NextButton {
                title: "Start the Import!",
                kind: "button",
                onclick: move |_| {
                    let new_params = params.get();
                    let mut new_config = config.clone();
                    new_config.set_crawl_options(new_params);
                    dbg!(&new_config);
                    loading_state.set(LoadingState::Loading(new_config))
                },
            }
            small {
                strong {"Note:"}
                "This can take a long time. Depending on your tweets, followers and lists, up to hours."
            }
        }
    }})
}
