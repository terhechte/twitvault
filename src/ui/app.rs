#![allow(non_snake_case)]
use std::cell::Cell;

use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::prelude::*;

use crate::config::Config;
use crate::storage::Storage;

use super::loading_component::LoadingComponent;
use super::login_component::LoginComponent;
use super::main_component::MainComponent;
use super::setup_component::SetupComponent;
use super::types::{LoadingState, StorageWrapper};

pub const TWATVAULT_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-box2-heart" viewBox="0 0 16 16"><path d="M8 7.982C9.664 6.309 13.825 9.236 8 13 2.175 9.236 6.336 6.31 8 7.982Z"/><path d="M3.75 0a1 1 0 0 0-.8.4L.1 4.2a.5.5 0 0 0-.1.3V15a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1V4.5a.5.5 0 0 0-.1-.3L13.05.4a1 1 0 0 0-.8-.4h-8.5Zm0 1H7.5v3h-6l2.25-3ZM8.5 4V1h3.75l2.25 3h-6ZM15 5v10H1V5h14Z"/></svg>"#;

pub fn run_ui(storage: Option<Storage>, config: Option<Config>) {
    dioxus::desktop::launch_with_props(
        App,
        AppProps {
            storage: Cell::new(storage),
            config: Cell::new(config),
        },
        |c| {
            c.with_window(default_menu)
                .with_window(|w| w.with_title("TwitVault"))
        },
    );
}

struct AppProps {
    storage: Cell<Option<Storage>>,
    config: Cell<Option<Config>>,
}

fn App(cx: Scope<AppProps>) -> Element {
    let loading_state = use_state(&cx, LoadingState::default);

    let storage: &UseState<Option<StorageWrapper>> = {
        let initial = cx.props.storage.take();
        use_state(&cx, || initial.map(StorageWrapper::new))
    };

    let config: &UseState<Option<Config>> = {
        let initial = cx.props.config.take();
        use_state(&cx, || initial)
    };

    let view = match (storage.get(), loading_state.get(), config.get()) {
        (Some(n), _, Some(c)) => cx.render(rsx!(div {
            MainComponent {
                storage: n.clone(),
                state: loading_state.clone(),
                config: c.clone()
            }
        })),
        (Some(_), _, None) => cx.render(rsx!(div {
            class: "alert",
            h3 {
                "Error:"
            }
            p {
                "Found storage but failed config "
                "You may need to re-login"
                button {
                    class: "btn btn-primary",
                    onclick: move |_| loading_state.set(LoadingState::default())
                }
            }
        })),
        (None, LoadingState::Login, _) => cx.render(rsx! {
            StartFlowContainer {
                LoginComponent {
                    loading_state: loading_state.clone()
                }
            }
        }),
        (None, LoadingState::Setup(config), _) => cx.render(rsx! {
            StartFlowContainer {
                SetupComponent {
                    config: config.clone(),
                    loading_state: loading_state.clone()
                }
            }
        }),
        (None, LoadingState::Loading(config), _) => cx.render(rsx! {
            StartFlowContainer {
                LoadingComponent {
                    config: config.clone(),
                    loading_state: loading_state.clone()
                }
            }
        }),
        (None, LoadingState::Loaded(wrapper, c), _) => {
            config.set(Some(c.clone()));
            storage.set(Some(wrapper.clone()));
            cx.render(rsx! {
                span {
                    // "Done"
                }
            })
        }
    };

    let is_loaded = storage.is_some();
    let main_class = if is_loaded {
        "overflow-hidden position-fixed w-auto"
    } else {
        "container"
    };

    let style_html = style_html();

    rsx!(cx, main {
        class: "{main_class}",
        div {
            dangerous_inner_html: "{style_html}"
        }
        is_loaded.then(|| rsx!(header {
            HeaderComponent {}
        })),

        view
    })
}

fn default_menu(builder: WindowBuilder) -> WindowBuilder {
    use dioxus::desktop::tao::menu::{MenuBar as Menu, MenuItem};
    let mut menu_bar_menu = Menu::new();
    let mut first_menu = Menu::new();
    first_menu.add_native_item(MenuItem::Copy);
    first_menu.add_native_item(MenuItem::Paste);
    first_menu.add_native_item(MenuItem::SelectAll);
    first_menu.add_native_item(MenuItem::CloseWindow);
    first_menu.add_native_item(MenuItem::Hide);
    first_menu.add_native_item(MenuItem::Quit);
    menu_bar_menu.add_submenu("TwitVault", true, first_menu);
    let s = LogicalSize::new(1080., 775.);
    builder
        .with_title("TwitVault")
        .with_menu(menu_bar_menu)
        .with_inner_size(s)
}

fn HeaderComponent(cx: Scope) -> Element {
    cx.render(rsx!(nav {
        class: "navbar navbar-expand-lg navbar-dark bg-dark",
        div {
            class: "container-fluid",
            span {
                class: "navbar-brand",
                i {
                    class: "bi",
                    dangerous_inner_html: "{TWATVAULT_ICON}"
                }
                small {
                    " TwitVault"
                }
            }
        }
    }))
}

#[derive(Props)]
struct StartFlowContainerProps<'a> {
    children: Element<'a>,
}

fn StartFlowContainer<'a>(cx: Scope<'a, StartFlowContainerProps<'a>>) -> Element {
    cx.render(rsx!(
        div {
            class: "px-4 my-5",
            &cx.props.children
        }
    ))
}

const fn style_html() -> &'static str {
    concat!(
        "<style>",
        include_str!("../assets/bootstrap.min.css"),
        "</style>"
    )
}
