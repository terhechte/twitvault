#![allow(non_snake_case)]

use dioxus::events::MouseEvent;
use dioxus::fermi::use_atom_state;
use dioxus::prelude::*;

use crate::search::{search, Description, Kind, Options, SearchResult};
use crate::ui::main_component::ColumnState;

use super::helpers::Spinner;
use super::main_component::COLUMN2;
use super::types::StorageWrapper;

#[inline_props]
pub fn SearchComponent(cx: Scope, storage: StorageWrapper) -> Element<'a> {
    let search_term = use_state(&cx, String::new);

    let filters = use_state(&cx, Options::default);

    let current_term = search_term.get().clone();
    let cloned = storage.clone();
    let options = filters.get().clone();
    let search_future = use_future(&cx, (), |_| async move {
        if current_term.is_empty() {
            return None;
        }
        Some(search(current_term.clone(), cloned.data(), options))
    });
    cx.render(rsx!(div {
        div {
            class: "mb-3",
            label {
                class: "form-label",
                "Search"
            }
            form {
                onsubmit: move |evt| {
                    search_term.set(evt.values["term"].to_string());
                    search_future.restart();
                },
                prevent_default: "onsubmit",
                input {
                    r#type: "text",
                    class: "form-control",
                    placeholder: "Search",
                    id: "term",
                    autocomplete: "off",
                    spellcheck: "false",
                    name: "term",
                }
                div {
                    class: "m-3",
                    Checkbox {
                        label: "Tweets",
                        name: "tweets"
                        checked: filters.tweets,
                        onclick: move |_| filters.set(filters.get().change(|s| s.tweets = !s.tweets))
                    }
                    Checkbox {
                        label: "Mentions",
                        name: "mentions"
                        checked: filters.mentions,
                        onclick: move |_| filters.set(filters.get().change(|s| s.mentions = !s.mentions))
                    }

                    Checkbox {
                        label: "Responses",
                        name: "responses"
                        checked: filters.profiles,
                        onclick: move |_| filters.set(filters.get().change(|s| s.responses = !s.responses))
                    }

                    Checkbox {
                        label: "Profiles",
                        name: "profiles"
                        checked: filters.profiles,
                        onclick: move |_| filters.set(filters.get().change(|s| s.profiles = !s.profiles))
                    }
                }
            }
            { match search_future.value() {
                Some(Some(v)) => rsx!(ResultListComponent {
                    data: v,
                }),
                Some(None) => rsx!(div {
                    class: "alert",
                    h3 {
                        "No results found"
                    }
                }),
                None => rsx!(Spinner {
                    title: "Searching".to_string()
                })
            }}
        }
    }))
}

#[inline_props]
fn Checkbox<'a>(
    cx: Scope,
    label: &'static str,
    name: &'static str,
    checked: bool,
    onclick: EventHandler<'a, MouseEvent>,
) -> Element {
    cx.render(rsx!(div {
        class: "form-check form-check-inline",
        input {
            class: "form-check-input",
            onclick: move |evt| onclick.call(evt),
            r#type: "checkbox",
            name: "{name}",
            checked: "{checked}",
            id: "{name}"
        }
        label {
            class: "form-check-label",
            r#for: "{name}",
            "{label}"
        }
    }))
}

#[derive(Props)]
pub struct ResultListProps<'a> {
    data: &'a [SearchResult],
}

pub fn ResultListComponent<'a>(cx: Scope<'a, ResultListProps>) -> Element<'a> {
    let column2 = use_atom_state(&cx, COLUMN2);
    let results_rendered = cx.props.data.iter().map(|r| {
        if let Some(desc) = r.desc.first() {
            let d = render_result(desc);
            match r.kind {
                Kind::Tweet(tweet) => {
                    rsx!(div {
                        class: "alert alert-light m-3",
                        h6 {
                            class: "alert-heading",
                            "Match in Tweet "
                            span {
                                class: "text-primary",
                                "{desc.field}"
                            }
                        }
                        p {
                            dangerous_inner_html: "{d}"
                        }
                        hr {}
                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            onclick: move |_| column2.set(ColumnState::AnyTweet(tweet)),
                            "Select"
                        }
                    })
                }
                Kind::Profile(profile) => {
                    rsx!(div {
                        class: "alert alert-light m-3",
                        h6 {
                            class: "alert-heading",
                            "Match in Profile: "
                            span {
                                class: "text-primary",
                                "{desc.field}"
                            }
                        }
                        p {
                            dangerous_inner_html: "{d}"
                        }
                        hr {}
                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            onclick: move |_| column2.set(ColumnState::Profile(profile)),
                            "Select"
                        }
                    })
                }
            }
        } else {
            rsx!(div {
                "No preview possible"
            })
        }
    });

    cx.render(rsx!(div {
        div {
            class: "vstack gap-3 p-3",
            h5 { "Search Results" }
            results_rendered
        }
    }
    ))
}

fn render_result(desc: &Description) -> String {
    let mut n = desc.content.to_string();
    for a in desc.highlights.iter().rev() {
        n.insert_str(a.end, "</b>");
        n.insert_str(a.start, "<b class='text-info'>");
    }
    n
}
