//! R3.2 (UI-SOD-4) — Directory service client for the Subject → Handles
//! admin page.
//!
//! `ck.find.directory.query.list_handles_for_subject` is the inverse of
//! `resolve_handle`: given a known holder/principal DID it returns the
//! currently visible signed `ck.schema.handle_claim.v1` evidence, after
//! the directory applies disclosure policy / issuer trust / audience /
//! Realm-intent filtering. The admin "Subject → Handles" operator view
//! calls this to triage which handles a subject is currently bound to.
//!
//! 404-tolerant: directories that pre-date arkret-spec @ b56cab1 don't
//! expose this endpoint yet — the page surfaces the error to the
//! operator rather than crashing.

use crate::api::client::api_client;
use crate::types::{DirectorySubjectHandleList, ListHandlesForSubjectRequest};
use crate::utils::net::error::HttpError;

/// `POST /_arkret/find/directory/list-handles-for-subject`.
pub const LIST_HANDLES_FOR_SUBJECT: &str = "/_arkret/find/directory/list-handles-for-subject";

/// Call `ck.find.directory.query.list_handles_for_subject`. The directory applies
/// disclosure / issuer-trust / audience / intent filtering server-side; callers
/// must still run [`DirectorySubjectHandleList::validate`] before rendering so
/// a response with mismatching `claims[].subject` fails closed.
pub async fn list_handles_for_subject(
    req: &ListHandlesForSubjectRequest,
) -> Result<DirectorySubjectHandleList, HttpError> {
    api_client(LIST_HANDLES_FOR_SUBJECT, "POST", Some(req)).await
}
