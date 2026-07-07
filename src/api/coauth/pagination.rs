use serde::de::DeserializeOwned;

use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

pub(crate) async fn get_jsonapi_first_page<T, U, F>(
    path: &str,
    per_page: u64,
    map: F,
) -> Result<PaginatedResponse<U>, HttpError>
where
    T: Default + DeserializeOwned,
    F: Fn(coauth_admin_types::SingleResource<T>) -> U,
{
    let per_page = per_page.max(1).to_string();
    let url = build_url(
        path,
        &[("page[first]", per_page.as_str()), ("count", "true")],
    )?;
    let resp: coauth_admin_types::PaginatedOutcome<T> = api_client(&url, "GET", NO_BODY).await?;
    Ok(flatten_jsonapi_page(resp, map))
}

pub(crate) fn flatten_jsonapi_page<T, U, F>(
    resp: coauth_admin_types::PaginatedOutcome<T>,
    map: F,
) -> PaginatedResponse<U>
where
    F: Fn(coauth_admin_types::SingleResource<T>) -> U,
{
    let data = resp
        .data
        .unwrap_or_default()
        .into_iter()
        .map(map)
        .collect::<Vec<_>>();
    let total = resp
        .meta
        .count
        .map(|count| count as u64)
        .unwrap_or(data.len() as u64);
    PaginatedResponse { data, total }
}

pub(crate) fn map_single_resource<T, U, F>(resp: coauth_admin_types::SingleOutcome<T>, map: F) -> U
where
    F: Fn(coauth_admin_types::SingleResource<T>) -> U,
{
    map(resp.data)
}
