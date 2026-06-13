use dioxus::prelude::*;

use crate::utils::i18n::t;
use crate::utils::net::telemetry;

/// Shared inline error banner used on list/detail pages.
///
/// 已迁移到 yoface:渲染交给 `yoface::ui::error_banner::ErrorBanner`(纯展示
/// css_module 组件)。本地保留这一薄封装作为**适配器**,承担 yoface 共享库
/// 刻意剥离的两项业务耦合(见 yoface README §3.2 接入说明):
///   * i18n —— 把 `common.error` / `common.retry` 文案与 `error.<errcode>` 本地化在 sodmin
///     侧解析后,经 `error_label` / `retry_label` / `detail` 传入 yoface 组件;
///   * telemetry —— banner 挂载时按 `errcode` fire-and-forget 上报埋点。
///
/// 对外签名与迁移前完全一致,因此 ~50 个调用点零改动。
#[component]
pub fn ErrorBanner(
    message: String,
    #[props(default)] errcode: Option<String>,
    #[props(default)] request_id: Option<String>,
    #[props(default)] retry_after_ms: Option<u64>,
    #[props(default)] on_retry: Option<EventHandler<MouseEvent>>,
) -> Element {
    let retry_label = t("common.retry");
    let error_label = t("common.error");

    // R3 P1 backfill (ENG-5) — fire-and-forget telemetry ping when the
    // banner mounts with a known wire errcode. No-op when telemetry is
    // disabled / the user has not opted in (see `utils::net::telemetry`).
    {
        let telemetry_code = errcode.clone();
        use_effect(move || {
            if let Some(code) = telemetry_code.as_deref() {
                telemetry::report_error(code, "ui.error_banner.shown");
            }
        });
    }

    // R3 (UI-7) — when the server returned a known wire code, look up the
    // localized copy via `error.<errcode>`. Falls back to None (no detail
    // line) when the key is missing.
    let detail = errcode.as_deref().and_then(|c| {
        let key = format!("error.{}", c);
        let v = t(&key);
        if v == key { None } else { Some(v) }
    });

    rsx! {
        yoface::ui::error_banner::ErrorBanner {
            message,
            error_label,
            retry_label,
            detail,
            errcode,
            request_id,
            retry_after_ms,
            on_retry,
        }
    }
}
