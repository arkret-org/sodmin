use dioxus::prelude::*;

use crate::utils::i18n::t;
use crate::utils::telemetry;

/// Shared inline error banner used on list/detail pages.
///
/// `message` is the human-readable summary. Pass optional `errcode`,
/// `request_id`, `retry_after_ms` from the `HttpError` envelope to
/// surface admin-grade context (errcode, request id, retry hint) below
/// the message. Pages that already have the [`HttpError`] in scope can
/// fill these from `e.body.as_ref().map(|b| b.errcode.clone())`,
/// `e.request_id.clone()`, and `e.retry_after_ms` respectively.
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
    let has_meta = errcode.is_some() || request_id.is_some() || retry_after_ms.is_some();
    // R3 P1 backfill (ENG-5) — fire-and-forget telemetry ping when the
    // banner mounts with a known wire errcode. No-op when telemetry is
    // disabled / the user has not opted in (see `utils::telemetry`).
    {
        let telemetry_code = errcode.clone();
        use_effect(move || {
            if let Some(code) = telemetry_code.as_deref() {
                telemetry::report_error(code, "ui.error_banner.shown");
            }
        });
    }
    // R3 (UI-7) — when the server returned a known wire code, look up
    // the localized copy via `error.<errcode>`. Falls back to the raw
    // wire code below (the chip) when the key is missing.
    let localized = errcode.as_deref().and_then(|c| {
        let key = format!("error.{}", c);
        let v = t(&key);
        if v == key { None } else { Some(v) }
    });
    rsx! {
        div { class: "rounded-md bg-destructive/10 p-4 space-y-2",
            div { class: "flex items-center justify-between gap-4",
                p { class: "text-sm text-destructive",
                    "{error_label}: {message}"
                }
                if let Some(handler) = on_retry {
                    button {
                        class: "text-sm font-medium text-primary hover:underline shrink-0",
                        onclick: move |evt| handler.call(evt),
                        "{retry_label}"
                    }
                }
            }
            if let Some(copy) = localized {
                p { class: "text-xs text-destructive/80",
                    "{copy}"
                }
            }
            if has_meta {
                div { class: "flex flex-wrap gap-2 text-xs text-muted-foreground",
                    if let Some(code) = errcode {
                        span { class: "rounded bg-destructive/10 px-2 py-0.5 font-mono",
                            "code: {code}"
                        }
                    }
                    if let Some(rid) = request_id {
                        span { class: "rounded bg-destructive/10 px-2 py-0.5 font-mono",
                            "ref: {rid}"
                        }
                    }
                    if let Some(retry_after_ms) = retry_after_ms {
                        span { class: "rounded bg-destructive/10 px-2 py-0.5 font-mono",
                            "retry in: {retry_after_ms / 1000}s"
                        }
                    }
                }
            }
        }
    }
}
