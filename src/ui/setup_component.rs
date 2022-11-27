#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf, rc::Rc};

use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::events::*;
use dioxus::fermi::{use_atom_state, AtomState};
use dioxus::prelude::*;
use egg_mode::user::TwitterUser;
use tokio::sync::mpsc::channel;
use tracing::warn;

use crate::config::{Config, RequestData};
use crate::crawler::DownloadInstruction;
use crate::storage::{Data, Storage, TweetId, UrlString, UserId};
use crate::types::Message;
use egg_mode::tweet::Tweet;

use super::types::LoadingState;

#[inline_props]
pub fn SetupComponent(cx: Scope, config: Config, loading_state: UseState<LoadingState>) -> Element {
    cx.render(rsx! { div {
        h4 {
            "Setup Config"
        }
        button {
            r#type: "button",
            class: "btn btn-primary",
            onclick: move |_| loading_state.set(LoadingState::Loading(config.clone())),
            "Next"
        }
    }})
}
