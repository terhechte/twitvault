#![allow(non_snake_case)]
use dioxus::{events::MouseEvent, prelude::*};

#[inline_props]
pub fn Spinner(cx: Scope, title: String) -> Element {
    cx.render(rsx!(div {
        class: "d-flex align-items-center alert alert-light",
        span {
            "{title}"
        }
        div {
            class: "spinner-border ms-auto",
        }
    }))
}

#[derive(Props)]
pub struct BoxProps<'a> {
    pub title: &'a str,
    pub children: Element<'a>,
}

pub fn Box<'a>(cx: Scope<'a, BoxProps<'a>>) -> Element {
    cx.render(rsx!(div {
        class: "border rounded p-4 bg-light",

        h3 {
            "{cx.props.title}"
        }
        &cx.props.children
    }))
}

#[derive(Props)]
pub struct NextButtonProps<'a> {
    pub title: &'a str,
    pub kind: &'a str,
    pub onclick: EventHandler<'a, MouseEvent>,
}

pub fn NextButton<'a>(cx: Scope<'a, NextButtonProps<'a>>) -> Element {
    cx.render(rsx!(div {
        div {
            class: "d-grid gap-2",
            button {
                r#type: "{cx.props.kind}",
                class: "btn btn-primary",
                onclick: move |evt| cx.props.onclick.call(evt),
                "{cx.props.title}"
            }
        }
    }))
}

#[derive(Props)]
pub struct CheckboxProps<'a> {
    pub name: &'a str,
    pub label: &'a str,
    pub onclick: EventHandler<'a, MouseEvent>,
    pub checked: bool,
    pub disabled: bool,
    pub children: Element<'a>,
}

pub fn Checkbox<'a>(cx: Scope<'a, CheckboxProps<'a>>) -> Element {
    let disabled = if cx.props.disabled { "true" } else { "false" };
    cx.render(rsx!(label {
            class: "list-group-item d-flex gap-2",
            input {
                onclick: move |evt| if !cx.props.disabled {
                    cx.props.onclick.call(evt)
                },
                class: "form-check-input flex-shrink-0",
                r#type: "checkbox",
                checked: "{cx.props.checked}",
                disabled: "{disabled}"
            }
            span {
                "{cx.props.name}"
                small {
                    class: "d-block text-muted",
                    "{cx.props.label}"
                }
            }
            &cx.props.children
        }
    ))
}

#[derive(Props)]
pub struct ShowMoreButtonProps<'a> {
    pub visible: bool,
    pub onclick: EventHandler<'a, MouseEvent>,
}

pub fn ShowMoreButton<'a>(cx: Scope<'a, ShowMoreButtonProps<'a>>) -> Element<'a> {
    if cx.props.visible {
        cx.render(rsx!( div {
            class: "d-grid gap-2",
            button {
                r#type: "button",
                class: "btn btn-primary",
                onclick: |n| cx.props.onclick.call(n),
                "Show More"
            }
        }
        ))
    } else {
        cx.render(rsx!(div {}))
    }
}

pub fn BottomSpacer(cx: Scope) -> Element {
    cx.render(rsx!(hr {
        style: "margin-bottom: 150px;"
    }))
}
