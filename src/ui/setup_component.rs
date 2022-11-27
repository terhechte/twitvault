#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::config::Config;

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
