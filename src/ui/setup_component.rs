#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::config::Config;

use super::types::LoadingState;

#[inline_props]
pub fn SetupComponent(cx: Scope, config: Config, loading_state: UseState<LoadingState>) -> Element {
    let params = use_state(&cx, move || config.crawl_options().clone());
    cx.render(rsx! { div {
        h4 {
            "Setup Config"
        }
        ul {
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.tweets = !o.tweets)),
                Entry {
                    label: "Your Feed",
                    checked: params.get().tweets,
                    disabled: false
                }
                ul {
                    li {
                        onclick: move |_| params.modify(|e| e.changed(|o| o.tweet_responses = !o.tweet_responses)),
                        Entry {
                            label: "Tweet Responses",
                            checked: params.get().tweet_responses,
                            disabled: !params.get().tweets
                        }
                    }
                    params.tweet_responses.then(|| {
                        rsx!(div {
                            class: "alert alert-primary",
                            "Tweet responses take a long time to load. Up to 3 hours per 1000 tweets"
                        })
                    })
                }
            }
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.tweet_profiles = !o.tweet_profiles)),
                Entry {
                    label: "User profiles from responses, mentions",
                    checked: params.get().tweet_profiles,
                    disabled: false
                }
            }
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.mentions = !o.mentions)),
                Entry {
                    label: "Your Mentions",
                    checked: params.get().mentions,
                    disabled: false
                }
            }
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.followers = !o.followers)),
                Entry {
                    label: "Your Followers",
                    checked: params.get().followers,
                    disabled: false
                }
            }
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.follows = !o.follows)),
                Entry {
                    label: "Your Follows",
                    checked: params.get().follows,
                    disabled: false
                }
            }
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.lists = !o.lists)),
                Entry {
                    label: "Lists and the profiles of the members",
                    checked: params.get().lists,
                    disabled: false
                }
            }
            li {
                onclick: move |_| params.modify(|e| e.changed(|o| o.media = !o.media)),
                Entry {
                    label: "Profile images and tweet media (images, videos)",
                    checked: params.get().media,
                    disabled: false
                }
            }
        }
        button {
            r#type: "button",
            class: "btn btn-primary",
            onclick: move |_| loading_state.set(LoadingState::Loading(config.clone())),
            "Next"
        }
    }})
}

#[inline_props]
fn Entry(cx: Scope, label: &'static str, checked: bool, disabled: bool) -> Element {
    let checked = if *checked { "checked" } else { " " };
    let disabled = if *disabled { "disabled" } else { " " };
    cx.render(rsx! { div {
        class: "form-check",
        input {
            class: "form-check-input {checked} {disabled}",
            r#type: "checkbox",
            id: "{label}"
        }
        label {
            class: "form-check-label",
            r#for: "{label}",
            "{label}"
        }
    }})
}
