#![allow(non_snake_case)]
use std::cell::Cell;

use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::prelude::*;

use crate::storage::Storage;

use super::loading_component::LoadingComponent;
use super::login_component::LoginComponent;
use super::main_component::MainComponent;
use super::setup_component::SetupComponent;
use super::types::{LoadingState, StorageWrapper};

pub fn run_ui(storage: Option<Storage>) {
    dioxus::desktop::launch_with_props(
        App,
        AppProps {
            storage: Cell::new(storage),
        },
        |c| {
            c.with_window(default_menu)
                .with_window(|w| w.with_title("My App"))
        },
    );
}

struct AppProps {
    storage: Cell<Option<Storage>>,
}

fn App(cx: Scope<AppProps>) -> Element {
    let loading_state = use_state(&cx, LoadingState::default);
    let initial = cx.props.storage.take();
    let storage: &UseState<Option<StorageWrapper>> =
        use_state(&cx, || initial.map(StorageWrapper::new));
    let view = match (storage.get(), loading_state.get()) {
        (Some(n), _) => cx.render(rsx!(div {
            MainComponent {
                storage: n.clone()
            }
        })),
        (None, LoadingState::Login) => cx.render(rsx! {
            StartFlowContainer {
                LoginComponent {
                    loading_state: loading_state.clone()
                }
            }
        }),
        (None, LoadingState::Setup(config)) => cx.render(rsx! {
            StartFlowContainer {
                SetupComponent {
                    config: config.clone(),
                    loading_state: loading_state.clone()
                }
            }
        }),
        (None, LoadingState::Loading(config)) => cx.render(rsx! {
            StartFlowContainer {
                LoadingComponent {
                    config: config.clone(),
                    loading_state: loading_state.clone()
                }
            }
        }),
        (None, LoadingState::Loaded(wrapper)) => {
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

    rsx!(cx, main {
        class: "{main_class}",
        link {
            href: "https://cdn.jsdelivr.net/npm/bootstrap@5.2.3/dist/css/bootstrap.min.css",
            rel: "stylesheet",
            crossorigin: "anonymous"
        },
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
    first_menu.add_native_item(MenuItem::CloseWindow);
    first_menu.add_native_item(MenuItem::Hide);
    first_menu.add_native_item(MenuItem::Quit);
    menu_bar_menu.add_submenu("My app", true, first_menu);
    let s = LogicalSize::new(1080., 775.);
    builder
        .with_title("Twittalypse")
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
                "My APp"
            }
            form {
                class: "d-flex",
                input {
                    class: "form-control form-control-sm me-2",
                    placeholder: "Search"
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
