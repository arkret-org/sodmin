//! P5 — minimal opt-in browser error telemetry.
//!
//! This module is intentionally tiny: when the deployment exposes a
//! `SODMIN_TELEMETRY_ENDPOINT` value in `/config.json` AND the operator
//! has flipped the local `sodmin_telemetry_opt_in` flag in localStorage,
//! browser-side errors (HTTP errors with a status, JSON parse failures,
//! capability denials) are POSTed to that endpoint as a structured
//! payload — no PII, no message bodies, only:
//!
//! * `errcode` (e.g. `cx.error.capability_denied`)
//! * `status` HTTP status code
//! * `request_id` from the `X-Cokret-Request-Id` header
//! * `path` (the request path, query string stripped)
//! * `ts` ISO-8601 timestamp
//!
//! Operators can correlate `request_id` with soland / coauth structured
//! logs to find the corresponding server-side log line. The
//! `request_id` is also preserved in error UI metadata so the operator
//! can paste it into a support ticket without opening DevTools.
//!
//! Sending failures are silently swallowed; telemetry MUST never break
//! the UI.
//!
//! TODO(P5-impl): exponential-backoff batching when the endpoint is
//! down; currently each call is a fire-and-forget single POST.

use serde::Serialize;

use crate::utils::error::HttpError;
use crate::utils::storage;

/// localStorage key for the operator opt-in flag.
const OPT_IN_KEY: &str = "sodmin_telemetry_opt_in";
/// localStorage key for the runtime telemetry endpoint URL. Populated
/// at boot from `/config.json#telemetry_endpoint`. May be unset.
const ENDPOINT_KEY: &str = "sodmin_telemetry_endpoint";

/// Set the endpoint URL — called from `pages::login` after
/// `load_runtime_config` reports a non-empty `telemetry_endpoint`.
pub fn set_endpoint(url: &str) {
    if url.trim().is_empty() {
        storage::remove_item(ENDPOINT_KEY);
    } else {
        storage::set_item(ENDPOINT_KEY, url.trim());
    }
}

/// Returns `true` when the operator opted into telemetry AND a runtime
/// endpoint is configured.
pub fn is_enabled() -> bool {
    storage::get_item(OPT_IN_KEY).as_deref() == Some("1")
        && storage::get_item(ENDPOINT_KEY)
            .map(|s| !s.is_empty())
            .unwrap_or(false)
}

#[derive(Debug, Clone, Serialize)]
struct ErrorEvent<'a> {
    errcode: &'a str,
    status: u16,
    request_id: Option<&'a str>,
    path: String,
    ts: String,
}

/// Reduce a concrete request URL to a low-cardinality, PII-free path
/// template for telemetry.
///
/// 1. Strips the query string + fragment (cursor tokens / filter values
///    count as PII).
/// 2. Templates inline id segments to `{id}`. Our REST paths embed
///    actor / account / realm / handle ids directly in the path (e.g.
///    `/_soland/admin/accounts/01HXY.../dids/did:web:x`), and those ids —
///    ULIDs, numeric ids, DIDs — are identifying. A segment is treated
///    as an id when it contains a `:` (DID / `ck:` ref), starts with a
///    digit (ULID / numeric id), or is an overly long opaque token.
///    Static words like `api`, `v1`, `accounts` are preserved.
fn redact_path(url: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    path.split('/')
        .map(|seg| {
            let is_id = seg.contains(':')
                || seg.chars().next().is_some_and(|c| c.is_ascii_digit())
                || seg.len() > 24;
            if is_id { "{id}" } else { seg }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Fire-and-forget POST of an error event. No-op when telemetry is
/// disabled, the URL is unset, or the user has not opted in.
///
/// Callers MUST NOT include the operator's input value, the server
/// error body, or anything that could carry credentials. Only the
/// fields documented on [`ErrorEvent`] are emitted.
pub fn report_http_error(path: &str, error: &HttpError) {
    if !is_enabled() {
        return;
    }
    let endpoint = match storage::get_item(ENDPOINT_KEY) {
        Some(e) if !e.is_empty() => e,
        _ => return,
    };

    let errcode = error
        .body
        .as_ref()
        .map(|b| b.errcode.as_str())
        .unwrap_or("cx.error.http_status");
    let event = ErrorEvent {
        errcode,
        status: error.status,
        request_id: error.request_id.as_deref(),
        path: redact_path(path),
        ts: iso_now(),
    };
    let body = match serde_json::to_string(&event) {
        Ok(b) => b,
        Err(_) => return,
    };

    // Fire-and-forget. We do not await — telemetry MUST never block.
    wasm_bindgen_futures::spawn_local(async move {
        // Telemetry MUST never break the UI: if the request builder
        // fails, drop the event silently rather than panicking.
        if let Ok(req) = gloo_net::http::Request::post(&endpoint)
            .header("Content-Type", "application/json")
            .body(body)
        {
            let _ = req.send().await;
        }
    });
}

/// Fire-and-forget reporter for browser-side errors that do NOT originate
/// from an HTTP envelope (panics, decoder failures, UI assertion misses).
///
/// Posts to the soland-side `/api/v1/telemetry/error` endpoint when the
/// admin runtime endpoint is configured (a future operator deployment may
/// route to that path directly; today the same `SODMIN_TELEMETRY_ENDPOINT`
/// sink is used). No-op when telemetry is disabled or the endpoint is unset.
///
/// `code` is a stable wire-style identifier (e.g. `ui.error_banner.shown`).
/// `context` is a short human-readable hint, MUST NOT include operator
/// input, secrets, or PII — keep it to component names and error kinds.
pub fn report_error(code: &str, context: &str) {
    if !is_enabled() {
        return;
    }
    let endpoint = match storage::get_item(ENDPOINT_KEY) {
        Some(e) if !e.is_empty() => e,
        _ => return,
    };

    // Prefer the canonical "/api/v1/telemetry/error" path when the
    // operator configured a host-only base URL; otherwise POST directly
    // to whatever the operator set.
    let target = if endpoint.ends_with("/telemetry/error") {
        endpoint
    } else if endpoint.ends_with('/') {
        format!("{endpoint}api/v1/telemetry/error")
    } else {
        format!("{endpoint}/api/v1/telemetry/error")
    };

    let payload = serde_json::json!({
        "code": code,
        "context": context,
        "ts": iso_now(),
    });
    let body = match serde_json::to_string(&payload) {
        Ok(b) => b,
        Err(_) => return,
    };

    // Fire-and-forget. We do not await — telemetry MUST never block.
    wasm_bindgen_futures::spawn_local(async move {
        // Telemetry MUST never break the UI: drop on builder failure.
        if let Ok(req) = gloo_net::http::Request::post(&target)
            .header("Content-Type", "application/json")
            .body(body)
        {
            let _ = req.send().await;
        }
    });
}

fn iso_now() -> String {
    // js_sys::Date::to_iso_string returns a JsString; convert to native.
    js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_path_strips_query_and_fragment() {
        assert_eq!(redact_path("/_cokret/self/x?cursor=c1"), "/_cokret/self/x");
        assert_eq!(redact_path("/_cokret/self/x"), "/_cokret/self/x");
        assert_eq!(redact_path("/_cokret/self/x?q=1#frag"), "/_cokret/self/x");
    }

    #[test]
    fn redact_path_templates_inline_ids() {
        // ULID account id + DID segment → both templated; static words
        // (_soland / admin / accounts / dids) preserved.
        assert_eq!(
            redact_path("/_soland/admin/accounts/01HXYABCDEF/dids/did:web:alice.example"),
            "/_soland/admin/accounts/{id}/dids/{id}"
        );
        // Numeric id templated.
        assert_eq!(
            redact_path("/_soland/admin/reports/12345"),
            "/_soland/admin/reports/{id}"
        );
        // No ids → unchanged.
        assert_eq!(
            redact_path("/_soland/admin/server/info"),
            "/_soland/admin/server/info"
        );
    }
}
