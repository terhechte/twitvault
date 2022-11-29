#![allow(non_snake_case)]

use dioxus::events::*;

use dioxus::prelude::*;
use tracing::warn;

use crate::config::{Config, RequestData};

use super::helpers::{Box, NextButton, Spinner};
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

impl Eq for LoginState {}

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
        (None, LoginState::Initial) => rsx!(Box {
            title: "Please Wait",
            Spinner {
                title: "Retrieving Login URL".to_string()
            }
        }),
        (Some(LoginStateResult::RequestData(n)), LoginState::Initial) => rsx!(Box {
            title: "Welcome to TwatVault",
            p {
                class: "lead",
                "This app will archive your Twitter account, including profiles, mentions and even responses."
            }
            p {
                "When you press the button, a browser window will open to give this app access to read your Twitter account"
            }
            p {
                "Once you give access, you will see a pin-code. Enter it here to proceed to the next step"
            }

            div {
                class: "card text-bg-light mb-3",
                div {
                    class: "card-header",
                    "Note:"
                }
                div {
                    class: "card-body",
                    p {
                        class: "card-text",
                        "The app is called "
                        strong {
                            "SwiftWatch "
                        }
                        "because I can't access the Twitter Dashboard. This is the only API Key I have"
                    }
                }
            }
            NextButton {
                title: "Next",
                kind: "button",
                onclick: move |_| {
                    if let Err(e) = webbrowser::open(&n.authorize_url) {
                        warn!("Could not open browser: {e:?}");
                    }
                    login_state.set(LoginState::LoadingPin(n.clone()));
                },
            }
        }),
        (Some(LoginStateResult::RequestData(n)), LoginState::LoadingPin(_)) => rsx!(Box {
            title: "Enter Pin",
            p {
                class: "lead",
                "Please enter the pin-code from your browser."
            }
            form {
                onsubmit: |evt: FormEvent| {
                    login_state.set(LoginState::EnteredPin(n.clone(), evt.values["pin"].to_string()));
                    state_machine.restart();
                },
                prevent_default: "onsubmit",

                div {
                    class: "vstack gap-3",
                    input {
                        class: "form-control",
                        "type": "text",
                        id: "pin",
                        name: "pin"
                    }

                    div {
                        class: "alert alert-info",
                        h6 {
                            "The following URL should have opened in a browser"
                        }
                        small {
                            a {
                                href: "{n.authorize_url}",
                                "{n.authorize_url}"
                            }
                        }
                    }

                    NextButton {
                        title: "Next",
                        kind: "submit",
                        onclick: move |_| { },
                    }
                }
            }
        }),
        (Some(LoginStateResult::LoggedIn(c)), LoginState::EnteredPin(_, _)) => rsx!(Box {
            title: "Success!",
            p {
                class: "lead",
                strong {
                    "Welcome {c.config_data.username} "
                }
                "In the next step, you can configure which data you wish to archive."
            }

            NextButton {
                title: "Next",
                kind: "button",
                onclick: move |_| {
                    loading_state.set(LoadingState::Setup(c.clone()));
                },
            }
        }),
        (Some(LoginStateResult::Error(e)), LoginState::LoadingPin(n)) => rsx!(Box {
            title: "Unknown Error"
            div {
                class: "alert alert-danger",
                "{e:?}"
            }
            NextButton {
                title: "Try Again",
                kind: "button",
                onclick: move |_| {
                    login_state.set(LoginState::LoadingPin(n.clone()));
                    state_machine.restart();
                },
            }
        }),
        (Some(LoginStateResult::Error(e)), LoginState::EnteredPin(n, _)) => rsx!(Box {
            title: "Invalid Pin",
            div {
                class: "alert alert-danger",
                "{e:?}"
            }
            NextButton {
                title: "Try Again",
                kind: "button",
                onclick: move |_| {
                    login_state.set(LoginState::LoadingPin(n.clone()));
                    state_machine.restart();
                },
            }
        }),
        (Some(LoginStateResult::Error(e)), _) => rsx!(Box {
            title: "Generic Error",
            div {
                class: "alert alert-danger",
                "{e:?}"
            }
            NextButton {
                title: "Try Again",
                kind: "button",
                onclick: move |_| {
                    login_state.set(LoginState::Initial);
                    state_machine.restart();
                },
            }
        }),
        _ => rsx!(div {
            Box {
                title: "Please Wait",
                Spinner {
                    title: "Loading".to_string()
                }
            }
        }),
    };

    cx.render(rsx! { div {
        ui
    }})
}
