#![allow(non_snake_case)]

use dioxus::{fermi::use_atom_state, prelude::*};

use crate::{storage::List, ui::main_component::ColumnState};

use super::main_component::COLUMN2;

#[derive(Props)]
pub struct ListListProps<'a> {
    lists: &'a [List],
}

pub fn ListListComponent<'a>(cx: Scope<'a, ListListProps>) -> Element<'a> {
    let lists_rendered = cx
        .props
        .lists
        .iter()
        .map(|list| cx.render(rsx!(ListComponent { list: list })));

    cx.render(rsx!(div {
        h5 { 
            style: "margin-top: 10px; margin-bottom: 5px; margin-left: 15px; font-weight: bold; color: slategray;",
            "Lists" 
        }
        lists_rendered
    }
    ))
}

#[derive(Props)]
struct ListProps<'a> {
    list: &'a List,
}

fn ListComponent<'a>(cx: Scope<'a, ListProps>) -> Element<'a> {
    let column2 = use_atom_state(&cx, COLUMN2);
    let name = &cx.props.list.name;
    let creator = &cx.props.list.list.user.screen_name;
    let creator_id = &cx.props.list.list.user.id;
    let subscribers = &cx.props.list.list.subscriber_count;
    let members = &cx.props.list.list.member_count;
    let description = &cx.props.list.list.description;

    let twitter_button = rsx!(a {
        class: "card-link",
        href: "https://twitter.com/{cx.props.list.list.uri}",
        "List on Twitter"
    });

    let twitter_button_a = rsx!(a {
        class: "card-link",
        href: "https://twitter.com/{creator}",
        "Creator on Twitter"
    });

    let open = rsx!(a {
        href: "#",
        class: "card-link",
        onclick: move |_| column2.set(ColumnState::List(cx.props.list.clone())),
        "Open"
    });

    cx.render(rsx!(div {
        class: "card",
        style: "margin-bottom: 10px",
        div {
            class: "card-body",
            h5 {
                class: "card-title",
                "{name}"
            }
            h6 {
                class: "card-subtitle mb-2 text-muted",
                "created by "
                a {
                    href: "#",
                    onclick: move |_| column2.set(ColumnState::Profile(*creator_id)),
                    "{creator}"
                }
            }
            p {
                class: "card-text",
                "{description}"
            }
            p {
                class: "card-text",
                "{subscribers} Subscribers, {members} Members"
            }
        }
        div {
            class: "card-footer",
            twitter_button
            twitter_button_a
            open
        }
    }))
}
