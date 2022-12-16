#![allow(non_snake_case)]
use std::{borrow::Cow, collections::HashMap};

use dioxus::prelude::*;
use egg_mode::user::TwitterUser;

use crate::config::Config;
use crate::storage::MediaResolver;

use super::helpers::{BottomSpacer, ShowMoreButton};
use super::user_component::AuthorComponent;

#[derive(Props)]
pub struct AuthorListProps<'a> {
    data: Cow<'a, [u64]>,
    media: MediaResolver<'a>,
    profiles: &'a HashMap<u64, TwitterUser>,
    label: String,
    config: &'a Config,
}

pub fn AuthorListComponent<'a>(cx: Scope<'a, AuthorListProps>) -> Element<'a> {
    let page_size = 100;
    let page = use_state(&cx, || page_size);
    let has_more = cx.props.data.len() > *page.get();
    let profiles_rendered = cx.props.data.iter().take(*page.get()).map(|id| {
        if let Some(user) = cx.props.profiles.get(id) {
            cx.render(rsx!(AuthorComponent {
                profile: user,
                media: cx.props.media.clone(),
                config: cx.props.config
            }))
        } else {
            cx.render(rsx!(div {
                "Could not find profile {id}"
            }))
        }
    });

    cx.render(rsx!(div {
        h5 {
            style: "margin-top: 10px; margin-bottom: 5px; margin-left: 15px; font-weight: bold; color: slategray;",
            "{cx.props.label}"
        }
        profiles_rendered
        ShowMoreButton {
            visible: has_more,
            onclick: move |_| page.set(page.get() + page_size)
        }
        BottomSpacer {}
    }
    ))
}
