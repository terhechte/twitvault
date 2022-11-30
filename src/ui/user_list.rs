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
    let profiles_rendered = cx.props.data.iter().map(|id| {
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
    }
    ))
}
