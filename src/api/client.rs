use gloo_net::http::{Headers, Request, RequestBuilder};
use serde::de::DeserializeOwned;
use web_sys::RequestCredentials;

use crate::utils::crypto::random_token;
use crate::utils::error::{AdminErrorEnvelope, HttpError, display_error};
use crate::utils::perf;

pub const HEADER_REQUEST_ID: &str = "X-Contrix-Request-Id";
pub const HEADER_IDEMPOTENCY_KEY: &str = "Idempotency-Key";

const SENSITIVE_QUERY_KEYS: &[&str] = &[
    "access_token",
    "auth",
    "authorization",
    "bearer",
    "id_token",
    "refresh_token",
    "session",
    "session_token",
    "token",
];

/// Per-request correlation id (12 bytes / ~16 chars base64url) used in the
/// `X-Contrix-Request-Id` header so admin actions can be traced across
/// proxy + coauth + soland logs.
pub fn generate_request_id() -> String {
    random_token(12)
}

/// Idempotency key for mutating admin operations. The protocol requires
/// strong uniqueness so retries collapse on the server side; 16 random
/// bytes (~22 base64url chars) gives ~128 bits of entropy.
pub fn generate_idempotency_key() -> String {
    format!("sodmin-{}", random_token(16))
}

pub async fn raw_fetch<T, F>(
    url: &str,
    method: &str,
    body: Option<String>,
    format_error: F,
) -> Result<T, HttpError>
where
    T: DeserializeOwned,
    F: Fn(u16, &str, Option<u64>) -> (String, Option<AdminErrorEnvelope>),
{
    let start_time = js_sys::Date::now();

    reject_query_credentials(url)?;

    let rid_value = generate_request_id();
    let rid = Some(rid_value.clone());

    // S5 token hardening: every admin API call rides the httpOnly
    // session cookie. There is no `Authorization: Bearer <token>`
    // header from the SPA — the cookie is sent automatically by the
    // browser when `credentials: "include"` is set on the request.
    let mut builder: RequestBuilder = match method {
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "PATCH" => Request::patch(url),
        "DELETE" => Request::delete(url),
        _ => Request::get(url),
    }
    .header("Accept", "application/json")
    .header(HEADER_REQUEST_ID, &rid_value)
    .credentials(RequestCredentials::Include);

    if is_mutation_method(method) {
        builder = builder.header(HEADER_IDEMPOTENCY_KEY, &generate_idempotency_key());
    }

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    let request = if let Some(body) = body {
        builder.body(body).map_err(|e| HttpError {
            message: e.to_string(),
            status: 0,
            body: None,
            request_id: rid.clone(),
            retry_after_ms: None,
        })?
    } else {
        builder.build().map_err(|e| HttpError {
            message: e.to_string(),
            status: 0,
            body: None,
            request_id: rid.clone(),
            retry_after_ms: None,
        })?
    };

    let response = request.send().await.map_err(|e| HttpError {
        message: e.to_string(),
        status: 0,
        body: None,
        request_id: rid.clone(),
        retry_after_ms: None,
    })?;

    let status = response.status();
    let response_request_id = header_value(response.headers(), HEADER_REQUEST_ID);
    let response_rid = response_request_id.or(rid.clone());
    let retry_after_ms = retry_after_ms(response.headers());
    let duration_ms = js_sys::Date::now() - start_time;
    perf::record_api_call(
        &redact_url_for_diagnostics(url),
        method,
        duration_ms,
        status,
    );

    if status == 204 {
        let empty = serde_json::from_str::<T>("{}").or_else(|_| serde_json::from_str::<T>("null"));
        return empty.map_err(|e| HttpError {
            message: e.to_string(),
            status,
            body: None,
            request_id: response_rid.clone(),
            retry_after_ms,
        });
    }

    let text = response.text().await.map_err(|e| HttpError {
        message: e.to_string(),
        status,
        body: None,
        request_id: response_rid.clone(),
        retry_after_ms,
    })?;

    if status >= 400 {
        let (message, error_body) = format_error(status, &text, retry_after_ms);
        let err = HttpError {
            message,
            status,
            body: error_body,
            request_id: response_rid,
            retry_after_ms,
        };
        // P5 — fire-and-forget opt-in telemetry. Internal no-op when
        // disabled / endpoint unset; never blocks the caller.
        crate::utils::telemetry::report_http_error(url, &err);
        return Err(err);
    }

    serde_json::from_str(&text).map_err(|e| HttpError {
        message: format!("JSON parse error: {e}"),
        status,
        body: None,
        request_id: response_rid,
        retry_after_ms,
    })
}

pub fn format_admin_error(
    status: u16,
    text: &str,
    retry_after_ms: Option<u64>,
) -> (String, Option<AdminErrorEnvelope>) {
    let mut error_body: Option<AdminErrorEnvelope> = serde_json::from_str(text).ok();
    if let Some(ref mut body) = error_body
        && body.retry_after_ms.is_none()
    {
        body.retry_after_ms = retry_after_ms;
    }
    let mut message = if let Some(ref eb) = error_body {
        display_error(&eb.errcode, status, eb.error.as_deref().unwrap_or(""))
    } else {
        display_error("cx.error.http_status", status, text)
    };
    let retry_after_ms = error_body
        .as_ref()
        .and_then(|body| body.retry_after_ms)
        .or(retry_after_ms);
    if let Some(retry_after_ms) = retry_after_ms {
        message.push_str(&format!(" Retry after {}s.", retry_after_ms / 1000));
    }
    (message, error_body)
}

pub async fn api_client<T: DeserializeOwned>(
    url: &str,
    method: &str,
    body: Option<String>,
) -> Result<T, HttpError> {
    // Spend a refresh round-trip ahead of the request when the cached
    // access token is within ~60s of expiry, so the happy path stays a
    // single call instead of failing with 401 and replaying.
    crate::api::auth::refresh_if_expiring_soon().await;

    let result = raw_fetch::<T, _>(url, method, body.clone(), format_admin_error).await;

    if let Err(ref err) = result
        && err.status == 401
        && crate::api::auth::handle_unauthorized().await
    {
        return raw_fetch::<T, _>(url, method, body, format_admin_error).await;
    }

    result
}

pub fn build_url(path: &str, params: &[(&str, &str)]) -> Result<String, HttpError> {
    reject_query_credentials(path)?;
    for (key, _) in params {
        reject_query_key(key)?;
    }

    let mut url = path.to_string();

    let query_params: Vec<String> = params
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect();

    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    Ok(url)
}

fn is_mutation_method(method: &str) -> bool {
    matches!(
        method.to_ascii_uppercase().as_str(),
        "POST" | "PUT" | "PATCH" | "DELETE"
    )
}

fn header_value(headers: Headers, name: &str) -> Option<String> {
    headers.get(name).filter(|value| !value.trim().is_empty())
}

fn retry_after_ms(headers: Headers) -> Option<u64> {
    header_value(headers, "Retry-After")?
        .trim()
        .parse::<u64>()
        .ok()
        .and_then(|seconds| seconds.checked_mul(1000))
}

fn reject_query_credentials(url: &str) -> Result<(), HttpError> {
    let Some(query) = url
        .split_once('?')
        .map(|(_, rest)| rest.split('#').next().unwrap_or(rest))
    else {
        return Ok(());
    };
    for pair in query.split('&') {
        let key = pair.split_once('=').map(|(key, _)| key).unwrap_or(pair);
        reject_query_key(key)?;
    }
    Ok(())
}

fn reject_query_key(key: &str) -> Result<(), HttpError> {
    let decoded = urlencoding::decode(key).unwrap_or_else(|_| key.into());
    if SENSITIVE_QUERY_KEYS.contains(&decoded.to_ascii_lowercase().as_str()) {
        return Err(HttpError::message(
            "query string authentication material is not allowed",
        ));
    }
    Ok(())
}

fn redact_url_for_diagnostics(url: &str) -> String {
    let Some((base, rest)) = url.split_once('?') else {
        return url.to_string();
    };
    let fragment = rest.split_once('#').map(|(_, fragment)| fragment);
    let query = rest.split('#').next().unwrap_or(rest);
    let redacted_query = query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            let decoded = urlencoding::decode(key).unwrap_or_else(|_| key.into());
            if SENSITIVE_QUERY_KEYS.contains(&decoded.to_ascii_lowercase().as_str()) {
                format!("{key}=REDACTED")
            } else if value.is_empty() {
                key.to_string()
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&");

    match fragment {
        Some(fragment) => format!("{base}?{redacted_query}#{fragment}"),
        None => format!("{base}?{redacted_query}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_url, format_admin_error, is_mutation_method, redact_url_for_diagnostics};

    #[test]
    fn build_url_rejects_query_credentials() {
        assert!(build_url("/admin/actors", &[("access_token", "secret")]).is_err());
        assert!(build_url("/admin/actors?token=secret", &[]).is_err());
    }

    #[test]
    fn build_url_appends_query_params() {
        assert_eq!(
            build_url("/admin/actors", &[("cursor", "c1")]).unwrap(),
            "/admin/actors?cursor=c1"
        );
    }

    #[test]
    fn diagnostics_redacts_sensitive_query_values() {
        assert_eq!(
            redact_url_for_diagnostics("/x?cursor=c1&access_token=secret#frag"),
            "/x?cursor=c1&access_token=REDACTED#frag"
        );
    }

    #[test]
    fn mutation_methods_get_idempotency_keys() {
        assert!(is_mutation_method("POST"));
        assert!(is_mutation_method("delete"));
        assert!(!is_mutation_method("GET"));
    }

    #[test]
    fn admin_error_preserves_retry_after() {
        let (_, body) = format_admin_error(
            429,
            r#"{"errcode":"cx.error.rate_limited","error":"slow down"}"#,
            Some(2000),
        );

        assert_eq!(body.unwrap().retry_after_ms, Some(2000));
    }
}
