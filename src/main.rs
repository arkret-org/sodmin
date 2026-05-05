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

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("./style.css")} }
        router::AppRouter {}
    }
}
