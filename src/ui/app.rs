#![allow(non_snake_case)]
use std::cell::Cell;

use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::use_window;
use dioxus::prelude::*;

use crate::config::Config;
use crate::storage::Storage;

use super::loading_component::LoadingComponent;
use super::login_component::LoginComponent;
use super::main_component::MainComponent;
use super::setup_component::SetupComponent;
use super::types::{LoadingState, StorageWrapper};

pub fn run_ui(storage: Option<Storage>, config: Option<Config>) {
    dioxus::desktop::launch_with_props(
        App,
        AppProps {
            storage: Cell::new(storage),
            config: Cell::new(config),
        },
        |c| {
            c.with_window(default_menu).with_window(|w| {
                #[cfg(target_os = "macos")]
                {
                    w.with_fullsize_content_view(true)
                        .with_titlebar_transparent(true)
                        .with_title_hidden(true)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    w.with_title("TwitVault")
                }
            })
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

    let desktop = use_window(&cx).clone();

    let script = r#"
    var script = document.createElement("script");
    script.type = "application/javascript";
    // loading the js into a `script` container did not work. So for now a link to the CDN
    script.src = "https://cdn.jsdelivr.net/npm/bootstrap@5.2.3/dist/js/bootstrap.bundle.min.js";
    document.head.appendChild(script);
    "#;

    use_future(&cx, (), |_| async move {
        time_sleep(1000).await;
        desktop.eval(script);
    });

    rsx!(cx, main {
        class: "{main_class}",
        view

        div {
            dangerous_inner_html: "{style_html}"
        }
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

async fn time_sleep(interval: usize) {
    tokio::time::sleep(tokio::time::Duration::from_millis(interval as u64)).await;
}
