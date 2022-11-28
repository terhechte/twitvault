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
    let message_state = use_state(&cx, || Message::Initial);

    let crawl = move |config: Config| {
        let (sender, mut receiver) = channel(256);
        cx.spawn(async move {
            let path = Config::archive_path();
            if let Err(e) = crate::crawler::crawl_new_storage(config, &path, sender).await {
                warn!("Error {e:?}");
            }
        });
        use_future(&cx, (), move |_| {
            let message_state = message_state.clone();
            let loading_state = loading_state.clone();
            async move {
                while let Some(msg) = receiver.recv().await {
                    let finished = match msg {
                        Message::Finished(o) => {
                            loading_state.set(LoadingState::Loaded(StorageWrapper::new(o)));
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
    };

    let ui = match message_state.get() {
        Message::Error(e) => rsx!(div {
                 "Error: {e:?}"
            }
        ),
        Message::Finished(_) => rsx!(div {
            // This should never appear here
        }),
        Message::Loading(msg) => rsx!(Box {
            title: "Importing"
            Spinner {
                title: ""
            }
            div {
                class: "lead",
                "{msg}"
            }
        }),
        Message::Initial => rsx!(div {
            button {
                r#type: "button",
                class: "btn btn-secondary",
                onclick: move |_| crawl(config.clone()),
                "Begin Crawling"
            }
        }),
    };
    cx.render(ui)
}
