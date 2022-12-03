#![allow(non_snake_case)]
use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use dioxus::prelude::*;
use egg_mode::user::TwitterUser;

use crate::storage::UrlString;

use super::user_component::AuthorComponent;

#[derive(Props)]
pub struct AuthorListProps<'a> {
    data: Cow<'a, [u64]>,
    media: &'a HashMap<UrlString, PathBuf>,
    profiles: &'a HashMap<u64, TwitterUser>,
    label: String,
}

pub fn AuthorListComponent<'a>(cx: Scope<'a, AuthorListProps>) -> Element<'a> {
    let page_size = 100;
    let page = use_state(&cx, || page_size);
    let profiles_rendered = cx.props.data.iter().take(*page.get()).map(|id| {
        if let Some(user) = cx.props.profiles.get(id) {
            cx.render(rsx!(AuthorComponent {
                profile: user,
                media: cx.props.media,
            }))
        } else {
            cx.render(rsx!(div {
                "Could not find profile {id}"
            }))
        }
    });

    cx.render(rsx!(div {
        h5 { "{cx.props.label}" }
        profiles_rendered
        div {
            class: "d-grid gap-2",
            button {
                r#type: "button",
                class: "btn btn-primary",
                onclick: move |_| page.set(page.get() + page_size),
                "Show More"
            }
        }
        hr {
            style: "margin-bottom: 150px;"
        }
    }
    ))
}
