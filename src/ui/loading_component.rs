#![allow(non_snake_case)]

use dioxus::prelude::*;

use tokio::sync::mpsc::channel;
use tracing::warn;

use crate::config::Config;

use crate::types::Message;

use super::helpers::{Box, Spinner};
use super::types::LoadingState;
use super::types::StorageWrapper;

#[inline_props]
pub fn LoadingComponent(
    cx: Scope,
    config: Config,
    loading_state: UseState<LoadingState>,
) -> Element {
    let appeared = cx.use_hook(|_| false);
    let message_state = use_state(&cx, || Message::Initial);

    let user_id = config.user_id();

    let (sender, mut receiver) = channel(4096);
    if !*appeared {
        *appeared = true;
        let cloned_config = config.clone();
        cx.spawn(async move {
            if let Err(e) = crate::crawler::crawl_new_storage(cloned_config, sender, user_id).await
            {
                warn!("Error {e:?}");
            }
        });
    }

    let label = if config.is_sync {
        "Syncing..."
    } else {
        "Importing..."
    };

    let cloned_config = config.clone();

    let future = use_future(&cx, (), move |_| {
        let message_state = message_state.clone();
        let loading_state = loading_state.clone();
        async move {
            while let Some(msg) = receiver.recv().await {
                let finished = match msg {
                    Message::Finished(o) => {
                        loading_state.set(LoadingState::Loaded(
                            StorageWrapper::new(o),
                            cloned_config.clone(),
                        ));
                        true
                    }
                    other => {
                        message_state.set(other);
                        false
                    }
                };
                if finished {
                    break;
                }
            }
        }
    });

    let ui = match message_state.get() {
        Message::Error(e) => rsx!(div {
            class: "alert alert-warning",
                h3 {
                    "Uh oh. Something went wrong",
                }
                "{e:?}"
            }
        ),
        Message::Finished(_) => rsx!(div {
            // This should never appear here
        }),
        Message::Loading(msg) => rsx!(div {
            class: "alert alert-info",
            h3 {
                "{label}"
            }
            Spinner {
                title: format!("{msg}")
            }
        }),
        Message::Initial => rsx!(div {
            class: "alert alert-info",
            h3 {
                "{label}"
            }
        }),
    };

    let value = match future.value() {
        Some(_) => "Done!",
        None => "Note: This can take a long time. Depending on your tweets, followers and lists, up to hours.",
    };

    cx.render(rsx!(Box {
        title: "Hard at Work",
        div {
            class: "card",
            div {
                class: "card-body",
                ui
                div {
                    class: "alert alert-info",
                    "{value}"
                }
            }
        }
    }))
}
