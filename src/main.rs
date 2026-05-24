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
    utils::i18n::sync_document_language();

    // S5: stamp the strict CSP into a `<meta http-equiv>` tag at
    // bootstrap. This is defense-in-depth — production deployments MUST
    // also set the matching `Content-Security-Policy` header at the
    // proxy / CDN tier so the browser receives the policy *before*
    // running any inline bootstrap. The meta tag catches mis-configured
    // proxies and dev-mode static hosting.
    install_csp_meta();

    dioxus::launch(App);
}

/// Install the `<meta http-equiv="Content-Security-Policy">` tag built
/// from `utils::csp::csp_meta_value()`. Idempotent: if the tag has
/// already been emitted (e.g. by the server-rendered shell) we leave
/// it alone — never overwrite a stricter policy.
fn install_csp_meta() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let head = match document.head() {
        Some(h) => h,
        None => return,
    };
    // Skip if a CSP meta already exists (server rendered shell).
    if let Ok(existing) = document.query_selector("meta[http-equiv=\"Content-Security-Policy\"]") {
        if existing.is_some() {
            return;
        }
    }
    let Ok(meta) = document.create_element("meta") else {
        return;
    };
    let _ = meta.set_attribute("http-equiv", "Content-Security-Policy");
    let _ = meta.set_attribute("content", &utils::csp::csp_meta_value());
    let _ = head.append_child(&meta);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("./style.css")} }
        router::AppRouter {}
    }
}
