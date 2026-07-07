//! coauth admin accounts: summary/detail, managed DID bindings, claims,
//! session grants and the risk-action lifecycle.

use serde::{Deserialize, Serialize};

use crate::api::client::{NO_BODY, NoBody, api_client, build_url};
use crate::types::CursorPage;
use crate::utils::net::error::HttpError;

const ACCOUNTS_PATH: &str = "/_coauth/admin/accounts";
const BRIDGE_DESCRIBE_PATH: &str = "/_coauth/admin/bridge/describe";
const COAUTH_INTEGRATION_DESCRIBE_PATH: &str = "/_coauth/account/integration/describe";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub primary_did: Option<String>,
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub is_deactivated: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub bridge_status: String,
}

impl CoauthAccountSummary {
    pub fn lifecycle_label(&self) -> &'static str {
        if self.is_deactivated {
            "Deactivated"
        } else if self.is_locked {
            "Locked"
        } else {
            "Active"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthManagedDidBinding {
    #[serde(default)]
    pub did: String,
    #[serde(default)]
    pub kind: CoauthDidBindingKind,
    #[serde(default)]
    pub state: CoauthDidBindingState,
    #[serde(default)]
    pub verification_status: CoauthDidBindingVerificationStatus,
    #[serde(default)]
    pub primary: bool,
    #[serde(default)]
    pub last_verified_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountClaim {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub claim_kind: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthSessionGrantSummary {
    #[serde(default)]
    pub grant_id: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthRiskActionHook {
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub approval_mode: String,
    #[serde(default)]
    pub todo: String,
}

pub use coauth_admin_types::{
    AdminBridgeDescribe as CoauthAdminBridgeDescribe, DidBindingKind as CoauthDidBindingKind,
    DidBindingState as CoauthDidBindingState,
    DidBindingVerificationStatus as CoauthDidBindingVerificationStatus,
    IntegrationManifest as CoauthIntegrationManifest,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountDetail {
    #[serde(default)]
    pub account: CoauthAccountSummary,
    #[serde(default)]
    pub managed_dids: Vec<CoauthManagedDidBinding>,
    #[serde(default)]
    pub claims: Vec<CoauthAccountClaim>,
    #[serde(default)]
    pub session_grants: Vec<CoauthSessionGrantSummary>,
    #[serde(default)]
    pub risk_action_current: CoauthAccountRiskActionCurrentState,
    #[serde(default)]
    pub risk_action_history: Vec<CoauthAccountRiskActionHistoryEntry>,
    #[serde(default)]
    pub risk_action_hook: CoauthRiskActionHook,
    #[serde(default)]
    pub admin_bridge: CoauthAdminBridgeDescribe,
    #[serde(default)]
    pub integration_manifest: CoauthIntegrationManifest,
}

pub use coauth_admin_types::{
    AccountRiskActionApprovalOutcome as CoauthAccountRiskActionApproval,
    AccountRiskActionApprovalRequestBody as CoauthAccountRiskActionApprovalDraft,
    AccountRiskActionCurrentOutcome as CoauthAccountRiskActionCurrentState,
    AccountRiskActionExecuteRequestBody as CoauthAccountRiskActionExecuteDraft,
    AccountRiskActionHistoryOutcome as CoauthAccountRiskActionHistoryEnvelopeShared,
    AccountRiskActionProposalOutcome as CoauthAccountRiskActionProposal,
    AccountRiskActionProposalRequestBody as CoauthAccountRiskActionDraft,
    AccountRiskActionTransitionRecord as CoauthAccountRiskActionHistoryEntry,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountRiskActionExecute {
    #[serde(default)]
    pub state_record_id: String,
    #[serde(default)]
    pub proposal_id: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub previous_state: String,
    #[serde(default)]
    pub execution_state: String,
    #[serde(default)]
    pub mutation_kind: String,
    #[serde(default)]
    pub state_revision: u64,
    #[serde(default)]
    pub transition_kind: String,
    #[serde(default)]
    pub executed_at: Option<String>,
    #[serde(default)]
    pub execution_mode: String,
    #[serde(default)]
    pub execution_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account:
        Option<coauth_admin_types::SingleOutcome<coauth_admin_types::AdminAccountAttributes>>,
    #[serde(default)]
    pub mutation_endpoint: String,
    #[serde(default)]
    pub allowed_next_transitions: Vec<String>,
    #[serde(default)]
    pub state_store_kind: String,
    #[serde(default)]
    pub todo: String,
}

type CoauthAccountClaimsEnvelope = coauth_admin_types::AdminAccountClaimsOutcome;

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAccountSessionGrantsEnvelope {
    #[serde(default)]
    data: Vec<CoauthSessionGrantSummary>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminPaginatedEnvelope<T> {
    #[serde(default)]
    data: Option<Vec<CoauthAdminResource<T>>>,
    #[serde(default)]
    meta: CoauthAdminPaginationMeta,
    #[serde(default)]
    links: coauth_admin_types::PaginationLinks,
}

type CoauthAdminSingleEnvelope<T> = coauth_admin_types::SingleOutcome<T>;
type CoauthAdminResource<T> = coauth_admin_types::SingleResource<T>;

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminPaginationMeta {
    #[serde(default)]
    count: Option<u64>,
}

type CoauthAdminAccountRecord = coauth_admin_types::AdminAccountAttributes;
type CoauthAdminDidBindingsEnvelope = coauth_admin_types::AdminAccountDidBindingsOutcome;
type CoauthAdminDidBindingRecord = coauth_admin_types::AdminAccountDidBinding;

#[derive(Debug, Clone, Serialize)]
struct AddAccountDidBindingRequestBody {
    did: String,
    kind: CoauthDidBindingKind,
    control_proof: ControlProofPayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    make_primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct ControlProofPayload {
    jws: String,
    nonce: String,
}

/// Filter inputs accepted by `list_accounts_cursor`. Empty strings are
/// dropped before encoding so the wire form only carries what the
/// operator actually filtered on.
#[derive(Debug, Clone, Default)]
pub struct AccountListFilter {
    pub handle: String,
    pub display_name: String,
}

/// Cursor-paginated account list. The caller passes back the opaque
/// `cursor` it received from the prior page's `links.next`.
///
/// Wire shape: `?filter[search]=…&filter[handle]=…&filter[display_name]=…
/// &cursor={base64url}&limit=N&count=true`. The base64url cursor is the
/// raw value coauth published; decoding/encoding on the wire is the
/// server's responsibility.
pub async fn list_accounts_cursor(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
    filter: &AccountListFilter,
) -> Result<CursorPage<CoauthAccountSummary>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("filter[search]", search),
        ("filter[handle]", filter.handle.trim()),
        ("filter[display_name]", filter.display_name.trim()),
        ("limit", limit_str.as_str()),
        ("count", "true"),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url(ACCOUNTS_PATH, &params)?;
    let resp: CoauthAdminPaginatedEnvelope<CoauthAdminAccountRecord> =
        api_client(&url, "GET", NO_BODY).await?;
    let summaries: Vec<CoauthAccountSummary> = resp
        .data
        .unwrap_or_default()
        .into_iter()
        .map(map_admin_account_summary_resource)
        .collect();
    let next_cursor = resp.links.next.as_deref().and_then(extract_cursor_param);
    Ok(CursorPage {
        data: summaries,
        next_cursor,
        total: resp.meta.count,
    })
}

pub async fn get_account_detail(id: &str) -> Result<CoauthAccountDetail, HttpError> {
    let bridge_url = BRIDGE_DESCRIBE_PATH;
    let integration_manifest_url = COAUTH_INTEGRATION_DESCRIBE_PATH;
    let summary_url = format!("/_coauth/admin/accounts/{}", urlencoding::encode(id));
    let dids_url = format!("/_coauth/admin/accounts/{}/dids", urlencoding::encode(id));
    let claims_url = format!("/_coauth/admin/accounts/{}/claims", urlencoding::encode(id));
    let grants_url = format!(
        "/_coauth/admin/accounts/{}/session-grants",
        urlencoding::encode(id)
    );
    let current_url = format!(
        "/_coauth/admin/accounts/{}/risk-action/current",
        urlencoding::encode(id)
    );
    let history_url = format!(
        "/_coauth/admin/accounts/{}/risk-action/history",
        urlencoding::encode(id)
    );
    let (summary, dids, claims, session_grants, bridge, integration_manifest, current, history): (
        CoauthAdminSingleEnvelope<CoauthAdminAccountRecord>,
        CoauthAdminDidBindingsEnvelope,
        CoauthAccountClaimsEnvelope,
        CoauthAccountSessionGrantsEnvelope,
        CoauthAdminBridgeDescribe,
        CoauthIntegrationManifest,
        CoauthAdminSingleEnvelope<CoauthAccountRiskActionCurrentState>,
        CoauthAccountRiskActionHistoryEnvelopeShared,
    ) = futures_util::try_join!(
        api_client(&summary_url, "GET", NO_BODY),
        api_client(&dids_url, "GET", NO_BODY),
        api_client(&claims_url, "GET", NO_BODY),
        api_client(&grants_url, "GET", NO_BODY),
        api_client(bridge_url, "GET", NO_BODY),
        api_client(integration_manifest_url, "GET", NO_BODY),
        api_client(&current_url, "GET", NO_BODY),
        api_client(&history_url, "GET", NO_BODY),
    )?;
    let account = map_admin_account_summary_resource(summary.data);
    Ok(CoauthAccountDetail {
        claims: claims
            .data
            .into_iter()
            .map(map_admin_account_claim)
            .collect(),
        session_grants: session_grants.data,
        risk_action_current: current.data.attributes,
        risk_action_history: history.data,
        managed_dids: dids.data.into_iter().map(map_admin_did_binding).collect(),
        account,
        risk_action_hook: CoauthRiskActionHook {
            endpoint: bridge.risk_action_path_template.replace("{account_id}", id),
            approval_mode: bridge.risk_action_approval_mode.clone(),
            todo: if bridge.todos.is_empty() {
                "Coauth account admin bridge advertises no outstanding operator follow-ups."
                    .to_string()
            } else {
                bridge.todos.join(" ")
            },
        },
        admin_bridge: bridge,
        integration_manifest,
    })
}

/// Add a managed DID binding to an account.
pub async fn add_account_did_binding(
    account_id: &str,
    did: &str,
    kind: CoauthDidBindingKind,
    proof_jws: &str,
    proof_nonce: &str,
) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/accounts/{}/dids",
        urlencoding::encode(account_id)
    );
    let body = AddAccountDidBindingRequestBody {
        did: did.to_owned(),
        kind,
        control_proof: ControlProofPayload {
            jws: proof_jws.to_owned(),
            nonce: proof_nonce.to_owned(),
        },
        make_primary: Some(kind == CoauthDidBindingKind::Primary),
    };
    let _: NoBody = api_client(&url, "POST", Some(&body)).await?;
    Ok(())
}

/// Remove a managed DID binding from an account. The DID is part of the
/// path so the request body is empty.
pub async fn remove_account_did_binding(account_id: &str, did: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/accounts/{}/dids/{}",
        urlencoding::encode(account_id),
        urlencoding::encode(did),
    );
    let _: NoBody = api_client(&url, "DELETE", NO_BODY).await?;
    Ok(())
}

/// Revoke a single claim by its record ULID.
pub async fn revoke_account_claim(claim_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/claims/{}/revoke",
        urlencoding::encode(claim_id),
    );
    let _: NoBody = api_client(&url, "POST", NO_BODY).await?;
    Ok(())
}

pub async fn submit_account_risk_action(
    id: &str,
    draft: &CoauthAccountRiskActionDraft,
) -> Result<CoauthAccountRiskActionProposal, HttpError> {
    let url = format!(
        "/_coauth/admin/accounts/{}/risk-action",
        urlencoding::encode(id)
    );
    // `CoauthAccountRiskActionDraft` is a re-export of
    // `coauth_admin_types::AccountRiskActionProposalRequestBody`, so the
    // draft is already the exact request-body shape coauth deserializes.
    api_client(&url, "POST", Some(draft)).await
}

pub async fn approve_account_risk_action(
    id: &str,
    proposal_id: &str,
    draft: &CoauthAccountRiskActionApprovalDraft,
) -> Result<CoauthAccountRiskActionApproval, HttpError> {
    let url = format!(
        "/_coauth/admin/accounts/{}/risk-action/{}/approve",
        urlencoding::encode(id),
        urlencoding::encode(proposal_id)
    );
    // `CoauthAccountRiskActionApprovalDraft` is a re-export of
    // `coauth_admin_types::AccountRiskActionApprovalRequestBody`.
    api_client(&url, "POST", Some(draft)).await
}

pub async fn execute_account_risk_action(
    id: &str,
    proposal_id: &str,
    draft: &CoauthAccountRiskActionExecuteDraft,
) -> Result<CoauthAccountRiskActionExecute, HttpError> {
    let url = format!(
        "/_coauth/admin/accounts/{}/risk-action/{}/execute",
        urlencoding::encode(id),
        urlencoding::encode(proposal_id)
    );
    // `CoauthAccountRiskActionExecuteDraft` is a re-export of
    // `coauth_admin_types::AccountRiskActionExecuteRequestBody`.
    api_client(&url, "POST", Some(draft)).await
}

fn map_admin_account_summary_resource(
    resource: CoauthAdminResource<CoauthAdminAccountRecord>,
) -> CoauthAccountSummary {
    let attributes = resource.attributes;
    let is_locked = attributes.status.is_locked();
    let is_deactivated = attributes.status.is_deactivated();
    let primary_did = attributes.effective_primary_did().map(str::to_owned);
    let bridge_status = if attributes.admin {
        "coauth_admin_accounts_v1+admin".to_string()
    } else {
        "coauth_admin_accounts_v1".to_string()
    };
    CoauthAccountSummary {
        id: resource.id,
        username: Some(attributes.handle),
        display_name: attributes.display_name,
        avatar_url: attributes.avatar_url,
        primary_did,
        is_locked,
        is_deactivated,
        created_at: attributes.created_at.map(|t| t.to_rfc3339()),
        updated_at: attributes.updated_at.map(|t| t.to_rfc3339()),
        bridge_status,
    }
}

fn map_admin_did_binding(binding: CoauthAdminDidBindingRecord) -> CoauthManagedDidBinding {
    CoauthManagedDidBinding {
        did: binding.did,
        kind: binding.kind,
        state: binding.state,
        verification_status: binding.verification_status,
        primary: binding.primary,
        last_verified_at: binding.last_verified_at.map(|t| t.to_rfc3339()),
    }
}

fn map_admin_account_claim(
    record: coauth_admin_types::AdminAccountClaimRecord,
) -> CoauthAccountClaim {
    CoauthAccountClaim {
        id: record.id,
        claim_kind: record.claim_kind,
        value: record.value,
        state: Some(record.state),
        source: Some(record.source),
    }
}

/// Parse the `cursor=…` query parameter out of a JSON:API `links.next` /
/// `links.prev` URL. Returns `None` when the link is absent or has no
/// cursor.
fn extract_cursor_param(link: &str) -> Option<String> {
    if link.is_empty() {
        return None;
    }
    let query = link.split_once('?').map(|(_, rest)| rest)?;
    let bare = query.split('#').next().unwrap_or(query);
    for pair in bare.split('&') {
        if let Some((key, value)) = pair.split_once('=')
            && (key == "cursor" || key == "page%5Bcursor%5D" || key == "page[cursor]")
        {
            let decoded = urlencoding::decode(value).ok()?.into_owned();
            if !decoded.is_empty() {
                return Some(decoded);
            }
        }
    }
    None
}

#[cfg(test)]
mod cursor_tests {
    use super::extract_cursor_param;

    #[test]
    fn extract_cursor_handles_plain_param() {
        assert_eq!(
            extract_cursor_param("/_coauth/admin/accounts?cursor=ABC123&limit=25"),
            Some("ABC123".to_string()),
        );
    }

    #[test]
    fn extract_cursor_handles_jsonapi_bracketed_param() {
        assert_eq!(
            extract_cursor_param("/_coauth/admin/accounts?page%5Bcursor%5D=DEF456"),
            Some("DEF456".to_string()),
        );
    }

    #[test]
    fn extract_cursor_returns_none_when_absent() {
        assert!(extract_cursor_param("/_coauth/admin/accounts?limit=25").is_none());
        assert!(extract_cursor_param("").is_none());
    }

    #[test]
    fn extract_cursor_decodes_url_escaping() {
        assert_eq!(
            extract_cursor_param("/x?cursor=A%3DB&limit=25"),
            Some("A=B".to_string()),
        );
    }
}
