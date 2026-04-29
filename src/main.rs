mod api;
mod components;
mod pages;
mod router;
mod types;
mod utils;

use dioxus::prelude::*;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");

    components::theme::apply_theme();

    if let Some(origin) = web_sys::window().and_then(|w| w.location().origin().ok()) {
        utils::storage::set_item("coauth_url", &origin);
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("./style.css")} }
        router::AppRouter {}
    }
}
