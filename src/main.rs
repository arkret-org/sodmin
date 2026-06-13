// `HttpError` is the canonical SPA error type and is wired through every
// `api/*` module's `Result<T, HttpError>` signature. Its size (a string
// message, optional envelope, optional retry hint) is intentional and
// boxing every call site would add noise without measurable benefit.
#![allow(clippy::result_large_err)]

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
/// from `utils::security::csp::csp_meta_value()`. Idempotent: if the tag has
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
    if let Ok(existing) = document.query_selector("meta[http-equiv=\"Content-Security-Policy\"]")
        && existing.is_some()
    {
        return;
    }
    let Ok(meta) = document.create_element("meta") else {
        return;
    };
    let _ = meta.set_attribute("http-equiv", "Content-Security-Policy");
    let _ = meta.set_attribute("content", &utils::security::csp::csp_meta_value());
    let _ = head.append_child(&meta);
}

#[component]
fn App() -> Element {
    rsx! {
        // 设计令牌注入顺序(任务 C4 步骤 4):
        //   1) yoface 默认令牌(第一层 shadcn 名 + 第二层别名 + 明暗开关)
        //   2) Soft Orbit 覆盖:重写 yoface 第二层别名为珊瑚橙完整色值,
        //      并把 sodmin 的 `.dark`/`.light` class 桥接到 yoface 明暗开关
        //   3) sodmin 既有 utility 系统(第一层名用 HSL 三元组,供 alpha 变体)
        // 详见 yoface_tokens.css 顶部说明。
        style { {yoface::TOKENS_CSS} }
        style { {include_str!("./yoface_tokens.css")} }
        style { {include_str!("./style.css")} }
        router::AppRouter {}
    }
}
