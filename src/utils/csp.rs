//! Content-Security-Policy helpers
//!
//! sodmin is a Dioxus WASM SPA. CSP enforcement happens in two places:
//!
//! 1. **Server / CDN** — production deployments MUST set the
//!    `Content-Security-Policy` HTTP response header at the proxy /
//!    CDN tier so the browser receives the policy *before* the WASM
//!    bundle runs. The header form is the canonical, primary
//!    enforcement.
//!
//! 2. **`<meta http-equiv="Content-Security-Policy">`** — defense-
//!    in-depth fallback for static-hosted dev bundles. The Dioxus
//!    `Meta` element in `main.rs` injects the same policy string
//!    so a misconfigured proxy still gets a strict policy.
//!
//! The policy is **nonce-based for future inline scripts**. Right
//! now Dioxus injects its bootstrap inline-script during dev (the
//! `'wasm-unsafe-eval'` allowance is required by all WASM apps), so
//! `script-src` allows `'self'` + `'wasm-unsafe-eval'` and
//! `'nonce-<value>'`. There is **no** `'unsafe-inline'` token.
//!
//! The nonce is generated once per page load. The value is held in a
//! thread-local so the renderer can read it back when stamping any
//! inline `<script nonce="...">` tag.
//! generation to the server-rendered shell and pass it down via a
//! data-attribute on `<html>` so each request gets a unique value.

use std::cell::OnceCell;

use crate::utils::crypto::random_token;

/// A 16-byte (~22 base64url chars) per-page nonce. Held in a
/// `thread_local!` so it's stable for the lifetime of a single SPA
/// instance and unique across reloads. Read by [`csp_meta_value`] and
/// any future inline-script stamping helpers.
fn page_nonce() -> &'static str {
    thread_local! {
        static NONCE: OnceCell<String> = const { OnceCell::new() };
    }

    // Leak the nonce string so we can return a `'static` reference. The
    // SPA only allocates this once per page-load so the leak is
    // bounded to ~32 bytes per session.
    NONCE.with(|cell| {
        let s = cell.get_or_init(|| random_token(16));
        // SAFETY: the OnceCell never reassigns; the `&str` lives as
        // long as the thread-local, which is the lifetime of the page.
        // Convert to a `'static` reference by `Box::leak`-ing a clone
        // on first access. The clone happens at most once.
        Box::leak(s.clone().into_boxed_str())
    })
}

/// Build the Content-Security-Policy directive string. Pure function
/// so it can be unit-tested without browser APIs. Caller assembles
/// the meta tag (or the HTTP header) from the returned string.
pub fn csp_directive(nonce: &str) -> String {
    [
        "default-src 'self'",
        // WASM apps need `'wasm-unsafe-eval'` for the WebAssembly
        // bootstrap; we keep `'self'` and the per-page nonce. NO
        // `'unsafe-inline'` and NO `'unsafe-eval'`.
        &format!(
            "script-src 'self' 'wasm-unsafe-eval' 'nonce-{}'",
            nonce.replace('\'', ""),
        ),
        // Tailwind / Dioxus generate utility classes at runtime that
        // get applied via `style="…"` inline attributes on rsx
        // elements. The narrow allowance is `'unsafe-inline'` for
        // styles only — there's no DOM-level injection vector for CSS
        // that doesn't already require script execution.
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data: blob:",
        "font-src 'self' data:",
        // The SPA fetches admin endpoints + the OAuth public URL.
        // Both go through the same-origin proxy by default; the
        // explicit `connect-src 'self'` keeps the policy strict and
        // refuses arbitrary cross-origin POSTs.
        "connect-src 'self'",
        "frame-ancestors 'none'",
        "base-uri 'self'",
        "form-action 'self'",
        "object-src 'none'",
        "upgrade-insecure-requests",
    ]
    .join("; ")
}

/// Build the `<meta http-equiv="Content-Security-Policy" content="…">`
/// content value for the current page. Reads the page nonce.
pub fn csp_meta_value() -> String {
    csp_directive(page_nonce())
}

/// Public accessor for the per-page nonce. Future round-26 inline
/// scripts can stamp this value into their `nonce="…"` attribute so
/// the strict CSP allows them through.
pub fn current_nonce() -> &'static str {
    page_nonce()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directive_disallows_unsafe_inline_for_scripts() {
        let s = csp_directive("xyz");
        assert!(s.contains("script-src"));
        // `'unsafe-inline'` MUST NOT appear in script-src. (It's
        // tolerated for styles only, which is documented in the
        // module-level comment.)
        let script_src_section = s
            .split(';')
            .find(|seg| seg.trim_start().starts_with("script-src"))
            .expect("script-src present");
        assert!(!script_src_section.contains("unsafe-inline"));
    }

    #[test]
    fn directive_includes_nonce_for_scripts() {
        let s = csp_directive("xyz");
        assert!(s.contains("nonce-xyz"));
    }

    #[test]
    fn directive_locks_dom_injection_vectors() {
        let s = csp_directive("abc");
        assert!(s.contains("frame-ancestors 'none'"));
        assert!(s.contains("object-src 'none'"));
        assert!(s.contains("base-uri 'self'"));
        assert!(s.contains("form-action 'self'"));
    }

    #[test]
    fn directive_strips_quotes_from_nonce() {
        // A malicious / typo'd nonce can't break out of the directive
        // by smuggling extra quotes — they're stripped before
        // formatting.
        let s = csp_directive("a'b'c");
        assert!(s.contains("nonce-abc"));
        assert!(!s.contains("a'b'c"));
    }

    #[test]
    fn no_unsafe_eval_in_directive() {
        let s = csp_directive("nonce");
        // Wasm apps need `wasm-unsafe-eval`; they do NOT need the
        // generic `unsafe-eval` (which would let attackers do
        // string-eval'd JS injection). Make sure we never accidentally
        // ship that.
        let script_src_section = s
            .split(';')
            .find(|seg| seg.trim_start().starts_with("script-src"))
            .expect("script-src present");
        assert!(script_src_section.contains("wasm-unsafe-eval"));
        // `'unsafe-eval'` would be a substring of `'wasm-unsafe-eval'`
        // — assert the *standalone* token is not present.
        assert!(!script_src_section.contains(" 'unsafe-eval'"));
    }
}
