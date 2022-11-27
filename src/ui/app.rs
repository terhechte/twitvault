#![allow(non_snake_case)]
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::prelude::*;

use super::loaded_component::LoadedComponent;
use super::loading_component::LoadingComponent;
use super::login_component::LoginComponent;
use super::setup_component::SetupComponent;
use super::types::LoadingState;

pub fn run_ui() {
    //dioxus::desktop::launch(App);
    // use dioxus::desktop::wry::application::window::WindowBuilder
    dioxus::desktop::launch_cfg(App, |c| {
        c.with_window(default_menu)
            .with_window(|w| w.with_title("My App"))
    });
}

fn App(cx: Scope) -> Element {
    let loading_state = use_state(&cx, LoadingState::default);
    let view = match loading_state.get() {
        LoadingState::Login => cx.render(rsx! {
            LoginComponent {
                loading_state: loading_state.clone()
            }
        }),
        LoadingState::Setup(config) => cx.render(rsx! {
            SetupComponent {
                config: config.clone(),
                loading_state: loading_state.clone()
            }
        }),
        LoadingState::Loading(config) => cx.render(rsx! {
            LoadingComponent {
                config: config.clone(),
                loading_state: loading_state.clone()
            }
        }),
        LoadingState::Loaded(store) => cx.render(rsx! {
            LoadedComponent {
                storage: store.clone()
            }
        }),
    };

    rsx!(cx, div {
        link {
            href: "https://cdn.jsdelivr.net/npm/bootstrap@5.2.3/dist/css/bootstrap.min.css",
            rel: "stylesheet",
            crossorigin: "anonymous"
        },
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
    builder.with_title("Twittalypse").with_menu(menu_bar_menu)
}
