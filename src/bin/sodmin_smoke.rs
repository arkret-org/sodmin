//! Post-deploy E2E smoke test runner for sodmin's soland admin surface.
//!
//! Hits the configured soland's admin health + each Stream H' endpoint
//! with an admin bearer token and prints a PASS/FAIL summary. Useful as
//! a one-shot ops sanity check after a deploy:
//!
//! ```ignore
//! cargo run --bin sodmin-smoke -- \
//!     --base-url https://soland.example.com \
//!     --token $SOLAND_ADMIN_TOKEN \
//!     --space-id cx:space:demo
//! ```
//!
//! Exit code is non-zero when any check fails so this can be wired into
//! a CI gate or deploy webhook. The binary intentionally does NOT import
//! the rest of the sodmin crate (which is wasm32-only); each request
//! body shape is reproduced as a small inline literal so the smoke
//! runner stays a self-contained native binary.

use std::env;
use std::process::ExitCode;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};

const DEFAULT_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Clone)]
struct Args {
    base_url: String,
    token: Option<String>,
    space_id: String,
    /// When true, treat 404 as a soft pass — the route may not yet be
    /// wired on this deployment (Stream H' is partially scaffolded). The
    /// summary marks these as `SKIP` and they don't count toward FAIL.
    tolerate_404: bool,
    timeout_secs: u64,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut base_url = env::var("SODMIN_SMOKE_BASE_URL").ok();
        let mut token = env::var("SODMIN_SMOKE_TOKEN")
            .ok()
            .or_else(|| env::var("SOLAND_ADMIN_TOKEN").ok());
        let mut space_id = env::var("SODMIN_SMOKE_SPACE_ID").ok();
        let mut tolerate_404 = env::var("SODMIN_SMOKE_TOLERATE_404")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
            .unwrap_or(true);
        let mut timeout_secs = DEFAULT_TIMEOUT_SECS;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--base-url" | "-u" => {
                    base_url = Some(
                        args.next()
                            .ok_or_else(|| "--base-url requires a value".to_string())?,
                    );
                }
                "--token" | "-t" => {
                    token = Some(
                        args.next()
                            .ok_or_else(|| "--token requires a value".to_string())?,
                    );
                }
                "--space-id" | "-s" => {
                    space_id = Some(
                        args.next()
                            .ok_or_else(|| "--space-id requires a value".to_string())?,
                    );
                }
                "--strict" => {
                    tolerate_404 = false;
                }
                "--tolerate-404" => {
                    tolerate_404 = true;
                }
                "--timeout" => {
                    let raw = args
                        .next()
                        .ok_or_else(|| "--timeout requires a value".to_string())?;
                    timeout_secs = raw.parse().map_err(|e| format!("invalid --timeout: {e}"))?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let base_url = base_url
            .ok_or_else(|| "--base-url (or SODMIN_SMOKE_BASE_URL env) is required".to_string())?;
        let space_id = space_id
            .ok_or_else(|| "--space-id (or SODMIN_SMOKE_SPACE_ID env) is required".to_string())?;

        Ok(Args {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            space_id,
            tolerate_404,
            timeout_secs,
        })
    }
}

fn print_help() {
    println!(
        "sodmin-smoke — soland admin endpoint smoke test\n\n\
USAGE:\n  sodmin-smoke --base-url <URL> --token <BEARER> --space-id <SPACE_ID>\n\n\
OPTIONS:\n  -u, --base-url     soland base URL (e.g. https://soland.example.com)\n  -t, --token        admin bearer token (also: $SOLAND_ADMIN_TOKEN)\n  -s, --space-id     Space id to probe (Stream H' is per-Space)\n      --strict       fail on 404 (default: skip — H' routes may not be wired)\n      --tolerate-404 treat 404 as SKIP (default)\n      --timeout      per-request timeout in seconds (default {DEFAULT_TIMEOUT_SECS})\n  -h, --help         print this message\n"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckOutcome {
    Pass,
    Fail,
    /// Endpoint not yet wired on this deployment (404 with --tolerate-404).
    Skip,
}

impl CheckOutcome {
    fn label(&self) -> &'static str {
        match self {
            CheckOutcome::Pass => "PASS",
            CheckOutcome::Fail => "FAIL",
            CheckOutcome::Skip => "SKIP",
        }
    }
}

/// Pure helper: classify an HTTP response status into a `CheckOutcome`.
/// 2xx is always PASS; 404 is SKIP iff tolerate_404; everything else
/// (including 4xx auth/validation and 5xx) is FAIL. Pulled out as a
/// pure function so the policy can be unit-tested without sending
/// requests.
pub(crate) fn classify_status(status: u16, tolerate_404: bool) -> CheckOutcome {
    if (200..300).contains(&status) {
        CheckOutcome::Pass
    } else if status == 404 && tolerate_404 {
        CheckOutcome::Skip
    } else {
        CheckOutcome::Fail
    }
}

/// Pure helper: build a per-Space admin URL like
/// `<base>/api/admin/v1/spaces/<id>/<suffix>`. URL-encodes the space id.
pub(crate) fn build_space_url(base_url: &str, space_id: &str, suffix: &str) -> String {
    format!(
        "{}/api/admin/v1/spaces/{}/{}",
        base_url.trim_end_matches('/'),
        urlencoding_encode(space_id),
        suffix.trim_start_matches('/'),
    )
}

/// Minimal URL-encoder for path segments. Native binary doesn't depend
/// on the `urlencoding` crate (which the wasm crate uses); this avoids a
/// duplicate dep gate just for path encoding. Encodes the characters
/// that conflict with path syntax (`/`, `?`, `#`) and unsafe / reserved
/// chars.
fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            b':' => {
                // Colon is allowed in URI path segments per RFC 3986
                // (pchar) and `cx:space:...` ids depend on it. Leaving
                // it un-encoded keeps the smoke output readable.
                out.push(':');
            }
            b => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
struct CheckResult {
    name: String,
    method: String,
    status: Option<u16>,
    outcome: CheckOutcome,
    detail: String,
}

fn run_check(client: &Client, name: &str, method: &str, url: &str, args: &Args) -> CheckResult {
    let req = match method {
        "GET" => client.get(url),
        "POST" => client.post(url).body("{}"),
        other => {
            return CheckResult {
                name: name.to_string(),
                method: other.to_string(),
                status: None,
                outcome: CheckOutcome::Fail,
                detail: format!("unsupported method: {other}"),
            };
        }
    };

    match req.send() {
        Ok(resp) => {
            let status = resp.status();
            let outcome = classify_status(status.as_u16(), args.tolerate_404);
            let detail = match outcome {
                CheckOutcome::Pass => "ok".to_string(),
                CheckOutcome::Skip => "404 — endpoint not yet wired".to_string(),
                CheckOutcome::Fail => format!(
                    "{} — {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("error")
                ),
            };
            CheckResult {
                name: name.to_string(),
                method: method.to_string(),
                status: Some(status.as_u16()),
                outcome,
                detail,
            }
        }
        Err(e) => CheckResult {
            name: name.to_string(),
            method: method.to_string(),
            status: None,
            outcome: CheckOutcome::Fail,
            detail: format!("transport error: {e}"),
        },
    }
}

fn run() -> ExitCode {
    let args = match Args::parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("sodmin-smoke: {e}");
            eprintln!("run with --help for usage");
            return ExitCode::from(2);
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    if let Some(token) = &args.token {
        match HeaderValue::from_str(&format!("Bearer {token}")) {
            Ok(hv) => {
                headers.insert(AUTHORIZATION, hv);
            }
            Err(e) => {
                eprintln!("sodmin-smoke: bad token: {e}");
                return ExitCode::from(2);
            }
        }
    }

    let client = match Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(args.timeout_secs))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sodmin-smoke: failed to build http client: {e}");
            return ExitCode::from(2);
        }
    };

    let base = &args.base_url;
    let space_id = &args.space_id;

    // Health probe + Stream H' GET endpoints. Mutating endpoints
    // (rotate-signing-key, partial-signature submit, anchorer/reconfigure,
    // bottom/repair, anchor-dag/compact, covered-frontier/advance) are
    // intentionally NOT exercised here — running them post-deploy would
    // mutate state. Smoke checks reachability + auth only.
    let checks = vec![
        ("admin/health", "GET", format!("{base}/api/admin/v1/health")),
        (
            "spaces/anchorer (H'1/H'2)",
            "GET",
            build_space_url(base, space_id, "anchorer"),
        ),
        (
            "spaces/anchor-dag (H'4)",
            "GET",
            build_space_url(base, space_id, "anchor-dag"),
        ),
        (
            "spaces/bottom (H'3)",
            "GET",
            build_space_url(base, space_id, "bottom"),
        ),
        (
            "spaces/consent (H'5)",
            "GET",
            build_space_url(base, space_id, "consent"),
        ),
        (
            "components (H'6)",
            "GET",
            format!("{base}/api/admin/v1/components"),
        ),
        (
            "spaces/covered-frontier (H'7)",
            "GET",
            build_space_url(base, space_id, "mls/covered-frontier"),
        ),
        (
            "spaces/anchorer/signing-key (H'8)",
            "GET",
            build_space_url(base, space_id, "anchorer/signing-key"),
        ),
        (
            "spaces/multisig/pending (H'9)",
            "GET",
            build_space_url(base, space_id, "multisig/pending"),
        ),
    ];

    println!("sodmin-smoke → {base} (space={space_id})");
    if args.token.is_none() {
        println!("  (no admin token supplied — auth-required endpoints will likely FAIL)");
    }
    println!();

    let mut results = Vec::with_capacity(checks.len());
    for (name, method, url) in checks {
        let result = run_check(&client, name, method, &url, &args);
        let status_label = result
            .status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        println!(
            "[{}] {:<6} {:<40} {:<5} {}",
            result.outcome.label(),
            result.method,
            result.name,
            status_label,
            result.detail,
        );
        results.push(result);
    }

    let pass = results
        .iter()
        .filter(|r| r.outcome == CheckOutcome::Pass)
        .count();
    let fail = results
        .iter()
        .filter(|r| r.outcome == CheckOutcome::Fail)
        .count();
    let skip = results
        .iter()
        .filter(|r| r.outcome == CheckOutcome::Skip)
        .count();

    println!();
    println!(
        "Summary: {pass} pass / {fail} fail / {skip} skip (total {})",
        results.len()
    );

    if fail > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    run()
}

#[cfg(test)]
mod tests {
    use super::{CheckOutcome, build_space_url, classify_status, urlencoding_encode};

    #[test]
    fn classify_status_buckets_by_code() {
        assert_eq!(classify_status(200, true), CheckOutcome::Pass);
        assert_eq!(classify_status(204, true), CheckOutcome::Pass);
        assert_eq!(classify_status(299, true), CheckOutcome::Pass);
        // 404 → Skip when tolerated, Fail otherwise.
        assert_eq!(classify_status(404, true), CheckOutcome::Skip);
        assert_eq!(classify_status(404, false), CheckOutcome::Fail);
        // 4xx that aren't 404 are always Fail (auth, validation, etc.).
        assert_eq!(classify_status(401, true), CheckOutcome::Fail);
        assert_eq!(classify_status(403, true), CheckOutcome::Fail);
        // 5xx is always Fail.
        assert_eq!(classify_status(500, true), CheckOutcome::Fail);
        assert_eq!(classify_status(503, true), CheckOutcome::Fail);
    }

    #[test]
    fn build_space_url_strips_trailing_base_slash() {
        let url = build_space_url("https://soland.example.com/", "cx:space:demo", "anchorer");
        assert_eq!(
            url,
            "https://soland.example.com/api/admin/v1/spaces/cx:space:demo/anchorer"
        );
    }

    #[test]
    fn build_space_url_handles_compound_suffix() {
        let url = build_space_url(
            "https://soland.example.com",
            "cx:space:demo",
            "mls/covered-frontier",
        );
        assert_eq!(
            url,
            "https://soland.example.com/api/admin/v1/spaces/cx:space:demo/mls/covered-frontier"
        );
    }

    #[test]
    fn urlencoding_encode_preserves_colon_and_alnum() {
        // `cx:space:01J9` is the typical id shape — colons MUST stay
        // unescaped or the URL becomes unreadable in logs.
        assert_eq!(urlencoding_encode("cx:space:01J9"), "cx:space:01J9");
        assert_eq!(urlencoding_encode("abc-123_~."), "abc-123_~.");
    }

    #[test]
    fn urlencoding_encode_escapes_unsafe_chars() {
        // Slashes and spaces must be percent-encoded so they don't
        // accidentally look like extra path components.
        assert_eq!(urlencoding_encode("a/b"), "a%2Fb");
        assert_eq!(urlencoding_encode("a b"), "a%20b");
        assert_eq!(urlencoding_encode("a?b"), "a%3Fb");
    }

    #[test]
    fn check_outcome_labels_match_summary() {
        assert_eq!(CheckOutcome::Pass.label(), "PASS");
        assert_eq!(CheckOutcome::Fail.label(), "FAIL");
        assert_eq!(CheckOutcome::Skip.label(), "SKIP");
    }
}
