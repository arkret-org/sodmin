use gloo_net::http::{Request, RequestBuilder};
use serde::de::DeserializeOwned;

use crate::utils::error::{HttpError, MatrixError, display_error};
use crate::utils::perf;
use crate::utils::storage;

pub fn generate_request_id() -> String {
    let now = js_sys::Date::now() as u64;
    let rand = (js_sys::Math::random() * 0xFFFF as f64) as u64;
    let combined = now.wrapping_mul(31).wrapping_add(rand);
    format!("{:08x}", combined & 0xFFFF_FFFF)
}

pub async fn raw_fetch<T, F>(
    url: &str,
    method: &str,
    body: Option<String>,
    format_error: F,
) -> Result<T, HttpError>
where
    T: DeserializeOwned,
    F: Fn(u16, &str) -> (String, Option<MatrixError>),
{
    let start_time = js_sys::Date::now();

    let rid = Some(generate_request_id());
    let token = storage::get_item("access_token");

    let mut builder: RequestBuilder = match method {
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "PATCH" => Request::patch(url),
        "DELETE" => Request::delete(url),
        _ => Request::get(url),
    }
    .header("Accept", "application/json");

    if let Some(ref token) = token {
        builder = builder.header("Authorization", &format!("Bearer {token}"));
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
        })?
    } else {
        builder.build().map_err(|e| HttpError {
            message: e.to_string(),
            status: 0,
            body: None,
            request_id: rid.clone(),
        })?
    };

    let response = request.send().await.map_err(|e| HttpError {
        message: e.to_string(),
        status: 0,
        body: None,
        request_id: rid.clone(),
    })?;

    let status = response.status();
    let duration_ms = js_sys::Date::now() - start_time;
    perf::record_api_call(url, method, duration_ms, status);

    if status == 204 {
        let empty = serde_json::from_str::<T>("{}").or_else(|_| serde_json::from_str::<T>("null"));
        return empty.map_err(|e| HttpError {
            message: e.to_string(),
            status,
            body: None,
            request_id: rid.clone(),
        });
    }

    let text = response.text().await.map_err(|e| HttpError {
        message: e.to_string(),
        status,
        body: None,
        request_id: rid.clone(),
    })?;

    if status >= 400 {
        let (message, error_body) = format_error(status, &text);
        return Err(HttpError {
            message,
            status,
            body: error_body,
            request_id: rid,
        });
    }

    serde_json::from_str(&text).map_err(|e| HttpError {
        message: format!("JSON parse error: {e}"),
        status,
        body: None,
        request_id: rid,
    })
}

pub fn format_admin_error(status: u16, text: &str) -> (String, Option<MatrixError>) {
    let error_body: Option<MatrixError> = serde_json::from_str(text).ok();
    let message = if let Some(ref eb) = error_body {
        display_error(&eb.errcode, status, eb.error.as_deref().unwrap_or(""))
    } else {
        display_error("M_INVALID", status, text)
    };
    (message, error_body)
}

pub async fn api_client<T: DeserializeOwned>(
    url: &str,
    method: &str,
    body: Option<String>,
) -> Result<T, HttpError> {
    let result = raw_fetch::<T, _>(url, method, body.clone(), format_admin_error).await;

    if let Err(ref err) = result {
        if err.status == 401 && crate::api::auth::handle_unauthorized().await {
            return raw_fetch::<T, _>(url, method, body, format_admin_error).await;
        }
    }

    result
}

pub fn build_url(path: &str, params: &[(&str, &str)]) -> Result<String, HttpError> {
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
