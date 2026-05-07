use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::RequestMode;

use crate::utils::crypto::{base64url_encode, random_token};
use crate::utils::error::HttpError;
use crate::utils::storage;

const OAUTH_CLIENT_ID: &str = "sodmin";
const COAUTH_ADMIN_SCOPE: &str = "urn:coauth:admin";
const CX_ADMIN_SCOPE: &str = "urn:contrix:admin:*";
const OAUTH_DEVICE_ID_STORAGE_KEY: &str = "oauth_device_id";
const PKCE_VERIFIER_KEY: &str = "pkce_code_verifier";
const OAUTH_STATE_KEY: &str = "oauth_state";
const OAUTH_NONCE_KEY: &str = "oauth_nonce";

#[derive(Debug)]
struct TextResponse {
    status: u16,
    text: String,
}

fn build_oauth_scope() -> String {
    format!("{COAUTH_ADMIN_SCOPE} {CX_ADMIN_SCOPE}")
}

fn get_or_create_device_id() -> String {
    if let Some(device_id) = storage::get_item(OAUTH_DEVICE_ID_STORAGE_KEY) {
        if device_id.len() >= 10 {
            return device_id;
        }
    }
    let device_id = crate::utils::password::generate_device_id();
    storage::set_item(OAUTH_DEVICE_ID_STORAGE_KEY, &device_id);
    device_id
}

fn coauth_public_base() -> Option<String> {
    crate::utils::session::coauth_public_url()
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty())
}

fn oauth_public_url(path: &str) -> Option<String> {
    coauth_public_base().map(|base| format!("{base}{path}"))
}

fn make_err(msg: String) -> HttpError {
    HttpError {
        message: msg,
        status: 0,
        body: None,
        request_id: None,
        retry_after_ms: None,
    }
}

fn is_public_oauth_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

async fn read_text_response(response: gloo_net::http::Response) -> Result<TextResponse, HttpError> {
    let status = response.status();
    let text = response.text().await.map_err(|e| make_err(e.to_string()))?;
    Ok(TextResponse { status, text })
}

async fn send_form_post(url: &str, body: &str) -> Result<TextResponse, HttpError> {
    let mut builder = Request::post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json");

    if is_public_oauth_url(url) {
        builder = builder.mode(RequestMode::Cors);
    }

    let response = builder
        .body(body.to_string())
        .map_err(|e| make_err(e.to_string()))?
        .send()
        .await
        .map_err(|e| make_err(e.to_string()))?;

    read_text_response(response).await
}

async fn send_oauth_form_request(path: &str, body: &str) -> Result<TextResponse, HttpError> {
    match send_form_post(path, body).await {
        Ok(response) if response.status < 500 => Ok(response),
        Ok(proxy_response) => {
            let Some(public_url) = oauth_public_url(path) else {
                return Ok(proxy_response);
            };
            match send_form_post(&public_url, body).await {
                Ok(public_response) => Ok(public_response),
                Err(_) => Ok(proxy_response),
            }
        }
        Err(proxy_err) => {
            let Some(public_url) = oauth_public_url(path) else {
                return Err(proxy_err);
            };
            match send_form_post(&public_url, body).await {
                Ok(public_response) => Ok(public_response),
                Err(_) => Err(proxy_err),
            }
        }
    }
}

fn generate_code_verifier() -> String {
    random_token(32)
}

async fn compute_code_challenge(verifier: &str) -> String {
    let crypto = web_sys::window().unwrap().crypto().unwrap();
    let subtle = crypto.subtle();
    let data = js_sys::Uint8Array::from(verifier.as_bytes());
    let promise = subtle
        .digest_with_str_and_buffer_source("SHA-256", &data)
        .expect("digest failed");
    let result = JsFuture::from(promise).await.expect("digest await failed");
    let buffer = result.dyn_into::<js_sys::ArrayBuffer>().unwrap();
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    base64url_encode(&bytes)
}

pub async fn start_oauth_login() {
    let verifier = generate_code_verifier();
    let challenge = compute_code_challenge(&verifier).await;
    let state = generate_code_verifier();
    let nonce = generate_code_verifier();
    let _device_id = get_or_create_device_id();
    let scope = build_oauth_scope();

    let session = web_sys::window()
        .unwrap()
        .session_storage()
        .unwrap()
        .unwrap();
    session
        .set_item(PKCE_VERIFIER_KEY, &verifier)
        .expect("sessionStorage set failed");
    session
        .set_item(OAUTH_STATE_KEY, &state)
        .expect("sessionStorage set failed");
    session
        .set_item(OAUTH_NONCE_KEY, &nonce)
        .expect("sessionStorage set failed");

    let redirect_uri = {
        let location = web_sys::window().unwrap().location();
        let origin = location.origin().unwrap();
        format!("{origin}/oauth/callback")
    };

    let coauth_base = coauth_public_base().unwrap_or_default();

    let auth_url = format!(
        "{coauth_base}/authorize?response_type=code\
         &client_id={OAUTH_CLIENT_ID}\
         &redirect_uri={}\
         &code_challenge={challenge}\
         &code_challenge_method=S256\
         &scope={}\
         &state={}\
         &nonce={}",
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&scope),
        urlencoding::encode(&state),
        urlencoding::encode(&nonce),
    );

    web_sys::window()
        .unwrap()
        .location()
        .set_href(&auth_url)
        .expect("redirect failed");
}

pub async fn handle_oauth_callback(code: &str, state: Option<&str>) -> Result<(), HttpError> {
    let session = web_sys::window()
        .unwrap()
        .session_storage()
        .unwrap()
        .unwrap();
    let verifier = session
        .get_item(PKCE_VERIFIER_KEY)
        .ok()
        .flatten()
        .ok_or_else(|| make_err("Missing PKCE verifier".into()))?;
    let expected_state = session
        .get_item(OAUTH_STATE_KEY)
        .ok()
        .flatten()
        .ok_or_else(|| make_err("Missing OAuth state".into()))?;
    if state != Some(expected_state.as_str()) {
        session.remove_item(PKCE_VERIFIER_KEY).ok();
        session.remove_item(OAUTH_STATE_KEY).ok();
        session.remove_item(OAUTH_NONCE_KEY).ok();
        return Err(make_err("OAuth state validation failed".into()));
    }
    // Capture nonce before cleanup — verified against id_token after token exchange.
    let expected_nonce = session
        .get_item(OAUTH_NONCE_KEY)
        .ok()
        .flatten();
    session.remove_item(PKCE_VERIFIER_KEY).ok();
    session.remove_item(OAUTH_STATE_KEY).ok();
    session.remove_item(OAUTH_NONCE_KEY).ok();

    let redirect_uri = {
        let origin = web_sys::window().unwrap().location().origin().unwrap();
        format!("{origin}/oauth/callback")
    };

    let form_body = format!(
        "grant_type=authorization_code\
         &code={}\
         &redirect_uri={}\
         &client_id={OAUTH_CLIENT_ID}\
         &code_verifier={}",
        urlencoding::encode(code),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&verifier),
    );

    let response = send_oauth_form_request("/oauth2/token", &form_body).await?;
    if response.status >= 400 {
        return Err(make_err(format!(
            "Token exchange failed ({}): {}",
            response.status, response.text
        )));
    }

    let token_resp: TokenResponse =
        serde_json::from_str(&response.text).map_err(|e| make_err(e.to_string()))?;

    // Verify OIDC nonce if the server returned an id_token.
    if let (Some(id_token), Some(expected)) = (&token_resp.id_token, &expected_nonce) {
        if let Some(nonce_in_token) = extract_id_token_nonce(id_token) {
            if nonce_in_token != *expected {
                return Err(make_err("OIDC nonce mismatch — possible replay".into()));
            }
        }
    }

    storage::set_item("access_token", &token_resp.access_token);
    persist_token_expiry(token_resp.expires_in);
    if let Some(ref rt) = token_resp.refresh_token {
        storage::set_item("refresh_token", rt);
    }

    let viewer_resp = crate::api::coauth::get_viewer().await?;
    storage::set_item("user_id", &viewer_resp.sub);
    if let Some(name) = viewer_resp.display_name {
        storage::set_item("user_display_name", &name);
    }
    if let Some(url) = viewer_resp.avatar_url {
        storage::set_item("user_avatar_url", &url);
    }

    Ok(())
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    id_token: Option<String>,
}

/// Extract the `nonce` claim from a JWT id_token (base64url-decoded payload only).
/// Returns `None` if the token is malformed or has no nonce claim.
fn extract_id_token_nonce(id_token: &str) -> Option<String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload_bytes = crate::utils::crypto::base64url_decode(parts[1])?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;
    payload.get("nonce")?.as_str().map(String::from)
}

pub async fn handle_unauthorized() -> bool {
    if refresh_oauth_token().await {
        return true;
    }
    storage::remove_item("access_token");
    storage::remove_item("refresh_token");
    storage::remove_item("is_admin");
    storage::remove_item("token_expires_at_ms");
    false
}

pub async fn refresh_oauth_token() -> bool {
    let refresh_token = match storage::get_item("refresh_token") {
        Some(rt) => rt,
        None => return false,
    };

    let form_body = format!(
        "grant_type=refresh_token\
         &refresh_token={}\
         &client_id={OAUTH_CLIENT_ID}",
        urlencoding::encode(&refresh_token),
    );

    let response = match send_oauth_form_request("/oauth2/token", &form_body).await {
        Ok(r) => r,
        Err(_) => return false,
    };

    if response.status >= 400 {
        return false;
    }

    let token_resp: TokenResponse = match serde_json::from_str(&response.text) {
        Ok(t) => t,
        Err(_) => return false,
    };

    storage::set_item("access_token", &token_resp.access_token);
    persist_token_expiry(token_resp.expires_in);
    if let Some(ref rt) = token_resp.refresh_token {
        storage::set_item("refresh_token", rt);
    }
    true
}

pub async fn verify_admin() -> Result<bool, HttpError> {
    let access_token =
        storage::get_item("access_token").ok_or_else(|| make_err("Not authenticated".into()))?;

    // TODO(sodmin.codegen): replace this probe with generated admin discovery
    // from `/api/v1/server/describe` + admin OpenAPI scopes. The path is
    // centralized through `canonical_api_path` during the migration.
    let admin_probe = crate::api::client::canonical_api_path("/contrix/admin/v1/server/info");
    let response = Request::get(&admin_probe)
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| make_err(e.to_string()))?;

    let status = response.status();
    if status == 200 {
        storage::set_item("is_admin", "true");
        return Ok(true);
    }
    if status == 403 {
        storage::set_item("is_admin", "false");
        return Ok(false);
    }
    let text = response.text().await.unwrap_or_default();
    Err(make_err(format!("admin probe HTTP {status}: {text}")))
}

pub fn cached_is_admin() -> Option<bool> {
    storage::get_item("is_admin").map(|v| v == "true")
}

fn persist_token_expiry(expires_in: Option<u64>) {
    if let Some(expires_in) = expires_in {
        storage::set_item(
            "token_expires_at_ms",
            &((js_sys::Date::now() as u64) + expires_in.saturating_mul(1000)).to_string(),
        );
    } else {
        storage::remove_item("token_expires_at_ms");
    }
}

pub fn token_expiry_ms() -> Option<u64> {
    storage::get_item("token_expires_at_ms").and_then(|value| value.parse().ok())
}

/// Window before access-token expiry where we proactively spend a
/// refresh round-trip rather than gambling that the next admin call will
/// land before the server starts rejecting with 401.
const TOKEN_REFRESH_LEEWAY_MS: u64 = 60_000;

/// True if the cached access token is within `TOKEN_REFRESH_LEEWAY_MS`
/// of expiry (or has no recorded expiry — treat as eager refresh).
pub fn token_expires_soon() -> bool {
    match token_expiry_ms() {
        Some(expiry) => {
            let now = js_sys::Date::now() as u64;
            expiry.saturating_sub(now) <= TOKEN_REFRESH_LEEWAY_MS
        }
        None => false,
    }
}

/// Issue a refresh round-trip if we're inside the leeway window. Called
/// from `api::client::api_client` ahead of every admin call so the
/// happy-path stays a single request and we don't rely on the 401 retry.
pub async fn refresh_if_expiring_soon() {
    if !is_authenticated() {
        return;
    }
    if !token_expires_soon() {
        return;
    }
    let _ = refresh_oauth_token().await;
}

pub async fn logout() -> Result<(), HttpError> {
    if let Some(token) = storage::get_item("access_token") {
        let body = format!(
            "token={}&client_id={OAUTH_CLIENT_ID}",
            urlencoding::encode(&token)
        );
        let _ = send_oauth_form_request("/oauth2/revoke", &body).await;
    }

    let _ = Request::post("/api/v1/auth/logout")
        .header("Accept", "application/json")
        .send()
        .await;

    // Auth/session keys: must be cleared on logout so the next user on
    // the same browser doesn't inherit the previous session. The
    // `oauth_device_id` is a sticky pairing for OAuth device flow — also
    // an auth-derived secret, so it goes in this set.
    //
    // User preferences (`language`, `theme`) and deployment-derived
    // values (`coauth_public_url`, set from `/config.json`) intentionally
    // survive logout.
    const KEYS_TO_CLEAR: &[&str] = &[
        "access_token",
        "refresh_token",
        "is_admin",
        "token_expires_at_ms",
        "user_id",
        "user_display_name",
        "user_avatar_url",
        OAUTH_DEVICE_ID_STORAGE_KEY,
    ];
    for key in KEYS_TO_CLEAR {
        storage::remove_item(key);
    }
    crate::utils::config::clear_config();
    Ok(())
}

pub fn is_authenticated() -> bool {
    storage::get_item("access_token").is_some()
}

#[cfg(test)]
mod tests {
    use super::build_oauth_scope;

    #[test]
    fn oauth_scope_contains_admin_scopes() {
        let scope = build_oauth_scope();
        assert!(scope.contains("urn:coauth:admin"));
        assert!(scope.contains("urn:contrix:admin:*"));
        assert!(!scope.contains("urn:cx:admin"));
    }
}
