#![allow(non_snake_case)]

use dioxus::events::*;

use dioxus::prelude::*;

use crate::config::{Config, RequestData};

use super::types::LoadingState;

#[derive(Clone)]
enum LoginState {
    Initial,
    LoadingPin(RequestData),
    EnteredPin(RequestData, String),
}

#[derive(Clone)]
enum LoginStateResult {
    RequestData(RequestData),
    LoggedIn(Config),
    Error(String),
}

impl PartialEq for LoginState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LoadingPin(_), Self::LoadingPin(_)) => true,
            (Self::EnteredPin(_, _), Self::EnteredPin(_, _)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[inline_props]
pub fn LoginComponent(cx: Scope, loading_state: UseState<LoadingState>) -> Element {
    let login_state = use_state(&cx, || LoginState::Initial);
    let current = (*login_state.current()).clone();

    let state_machine = use_future(&cx, login_state, move |login_state| {
        let current = (*login_state.current()).clone();
        async move {
            match current {
                LoginState::Initial => RequestData::request()
                    .await
                    .map(LoginStateResult::RequestData)
                    .unwrap_or_else(|e| LoginStateResult::Error(e.to_string())),
                LoginState::EnteredPin(data, pin) => data
                    .validate(&pin)
                    .await
                    .map(LoginStateResult::LoggedIn)
                    .unwrap_or_else(|e| LoginStateResult::Error(e.to_string())),
                LoginState::LoadingPin(n) => LoginStateResult::RequestData(n),
            }
        }
    });

    let ui = match (state_machine.value(), current) {
        (None, LoginState::Initial) => rsx!(div {
            "Retrieving Login URL"
        }),
        (Some(LoginStateResult::RequestData(n)), LoginState::Initial) => rsx!(div {
            a {
                class: "btn btn-primary",
                href: "{n.authorize_url}",
                onclick: move |_| {
                    login_state.set(LoginState::LoadingPin(n.clone()));
                },

                "Click here to login"
            }
        }),
        (Some(LoginStateResult::RequestData(n)), LoginState::LoadingPin(_)) => rsx!(div {
            h5 { "Please enter Pin"}
            form {
                onsubmit: |evt: FormEvent| {
                    login_state.set(LoginState::EnteredPin(n.clone(), evt.values["pin"].to_string()));
                    state_machine.restart();
                },
                prevent_default: "onsubmit",

                input { "type": "text", id: "pin", name: "pin" }

                button {
                    r#type: "submit",
                    class: "btn btn-primary",

                    "Next"
                }

            }
        }),
        (Some(LoginStateResult::LoggedIn(c)), LoginState::EnteredPin(_, _)) => rsx!(div {
            "Successfully logged in"
            button {
                r#type: "button",
                class: "btn btn-primary",
                onclick: move |_| {
                    loading_state.set(LoadingState::Setup(c.clone()));
                },

                "Next"
            }
        }),
        (Some(LoginStateResult::Error(e)), LoginState::LoadingPin(_)) => rsx!(div {
            "Could not gerate URL: {e:?}"
        }),
        (Some(LoginStateResult::Error(e)), LoginState::EnteredPin(n, _)) => rsx!(div {
            "Invalid Pin: {e:?} Please try again?"
            button {
                r#type: "button",
                class: "btn btn-primary",
                onclick: move |_| {
                    login_state.set(LoginState::LoadingPin(n.clone()));
                    state_machine.restart();
                },

                "Try Again"
            }
        }),
        (Some(LoginStateResult::Error(e)), _) => rsx!(div {
            "Error: {e:?} Please try again?"
            button {
                r#type: "button",
                class: "btn btn-primary",
                onclick: move |_| {
                    login_state.set(LoginState::Initial);
                    state_machine.restart();
                },

                "Try Again"
            }
        }),
        _ => rsx!(div {
            "Waiting"
        }),
    };

    cx.render(rsx! { div {
        ui
    }})
}
