use dioxus::{events::MouseEvent, prelude::*};

#[inline_props]
pub fn Spinner(cx: Scope, title: &'static str) -> Element {
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
    // pub on_click: EventHandler<'a, MouseEvent>,
    // pub href: String,
    pub children: Element<'a>,
}

pub fn Box<'a>(cx: Scope<'a, BoxProps<'a>>) -> Element {
    // let href = if cx.props.href.is_empty() {
    //     "javascript:void(0)"
    // } else {
    //     &cx.props.href
    // };
    // let href = cx.props.href.unwrap_or("javascript:void(0)");
    cx.render(rsx!(div {
        style: "width: 540px; padding: 8px;",
        h2 {
            class: "mt-5",
            "{cx.props.title}"
        }
        &cx.props.children
        // div {
        //     class: "d-grid gap-2",
        //     a {
        //         class: "btn btn-primary",
        //         href: "{href}",
        //         onclick: move |evt| cx.props.on_click.call(evt),
        //         "Next"
        //     }
        // }
    }))
}
