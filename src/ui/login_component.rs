#![allow(non_snake_case)]

use dioxus::events::*;

use dioxus::prelude::*;

use crate::config::{Config, RequestData};

use super::helpers::{Box, Spinner};
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
            Spinner {
                title: "Retrieving Login URL"
            }
        }),
        (Some(LoginStateResult::RequestData(n)), LoginState::Initial) => rsx!(Box {
            title: "WWelcome",
            // on_click: move |_| {
            //     login_state.set(LoginState::LoadingPin(n.clone()));
            // }
            // href: n.authorize_url.clone(),
            p {
                class: "lead",
                "This app will archive your Twitter account, including profiles, mentions and even responses."
            }
            p {
                "When you press the button, a browser window will open to give this app access to read your Twitter account"
            }
            p {
                "The app is called SwiftWatch because I can't access the Twitter Dashboard. This is the only API Key I have"
            }
            p {
                "Once you give access, you will see a pin-code. Enter it here to proceed to the next step"
            }
            div {
                class: "d-grid gap-2",
                a {
                    class: "btn btn-primary",
                    href: "{n.authorize_url}",
                    onclick: move |_| {
                        login_state.set(LoginState::LoadingPin(n.clone()));
                    },

                    "Next"
                }
            }
        }),
        (Some(LoginStateResult::RequestData(n)), LoginState::LoadingPin(_)) => rsx!(Box {
            title: "Enter Pin",
            p {
                class: "lead",
                "Please enter the pin-code from your browser"
            }
            form {
                onsubmit: |evt: FormEvent| {
                    login_state.set(LoginState::EnteredPin(n.clone(), evt.values["pin"].to_string()));
                    state_machine.restart();
                },
                prevent_default: "onsubmit",

                input { "type": "text", id: "pin", name: "pin" }

                div {
                    class: "d-grid gap-2",
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",

                        "Next"
                    }
                }
            }
        }),
        (Some(LoginStateResult::LoggedIn(c)), LoginState::EnteredPin(_, _)) => rsx!(Box {
            title: "Logged In",
            p {
                class: "lead",
                "In the next step, you can configure which data you wish to archive"
            }
            div {
                class: "d-grid gap-2",
                button {
                    r#type: "button",
                    class: "btn btn-primary",
                    onclick: move |_| {
                        loading_state.set(LoadingState::Setup(c.clone()));
                    },

                    "Next"
                }
            }
        }),
        (Some(LoginStateResult::Error(e)), LoginState::LoadingPin(n)) => rsx!(Box {
            title: "Unknown Error"
            div {
                class: "alert alert-danger",
                "{e:?}"
            }
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
        (Some(LoginStateResult::Error(e)), LoginState::EnteredPin(n, _)) => rsx!(Box {
            title: "Invalid Pin",
            div {
                class: "alert alert-danger",
                "{e:?}"
            }
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
        (Some(LoginStateResult::Error(e)), _) => rsx!(Box {
            title: "Generic Error",
            div {
                class: "alert alert-danger",
                "{e:?}"
            }
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
