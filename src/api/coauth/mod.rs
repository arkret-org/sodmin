//! coauth admin API client surface.
//!
//! Wire shapes that already live in the shared `coauth-admin-types`
//! crate are imported / re-exported from there. Anything still defined
//! inline here is on the migration list — when the corresponding coauth
//! admin handler graduates to a typed response, lift the struct into
//! `coauth-admin-types` and turn the local copy into a re-export.
//!
//! This module is split by domain into submodules; everything is
//! re-exported here so the historical flat `crate::api::coauth::X` paths
//! keep compiling unchanged.

mod accounts;
mod audit_feed;
mod connector_health;
mod notifications;
mod registration;
mod sessions;
mod upstream;
mod viewer;

// Generic pagination value types live in `crate::types` (shared across
// pages). Re-exported here so the legacy `crate::api::coauth::{PaginatedResponse,
// CursorPage}` paths stay valid. NOTE: these are intentionally distinct
// from the JSON:API envelope `coauth_admin_types::PaginatedResponse`
// (`{data, meta, links}`) and from `crate::types::api::ListResponse`
// (`{data, total, next_cursor}`) — the wire shapes differ field-for-field.
pub use crate::types::{CursorPage, PaginatedResponse};

pub use accounts::{
    add_account_did_binding, approve_account_risk_action, execute_account_risk_action,
    get_account_detail, list_accounts_cursor, remove_account_did_binding, revoke_account_claim,
    submit_account_risk_action, AccountListFilter, CoauthAccountClaim, CoauthAccountDetail,
    CoauthAccountRiskActionApproval, CoauthAccountRiskActionApprovalDraft,
    CoauthAccountRiskActionCurrentState, CoauthAccountRiskActionDraft,
    CoauthAccountRiskActionExecute, CoauthAccountRiskActionExecuteDraft,
    CoauthAccountRiskActionHistoryEntry, CoauthAccountRiskActionHistoryEnvelopeShared,
    CoauthAccountRiskActionProposal, CoauthAccountSummary, CoauthAdminBridgeDescribe,
    CoauthIntegrationManifest, CoauthManagedDidBinding, CoauthRiskActionHook,
    CoauthSessionGrantSummary,
};
pub use audit_feed::{list_audit_feed, AuditFeedFilter, CoauthAuditEntry};
pub use connector_health::{get_connector_health, CoauthConnectorHealth};
pub use notifications::{
    list_notification_channels, list_notification_templates, CoauthNotificationChannel,
    CoauthNotificationTemplate,
};
pub use registration::{
    create_registration_token, list_registration_tokens, revoke_registration_token,
    CoauthRegistrationToken,
};
pub use sessions::{
    create_personal_session, finish_oauth2_session, list_oauth2_sessions, list_personal_sessions,
    regenerate_personal_session, revoke_personal_session, CoauthOAuth2Session,
    CoauthPersonalSession,
};
pub use upstream::{
    create_upstream_provider, delete_upstream_link, delete_upstream_provider, list_upstream_links,
    list_upstream_providers, toggle_upstream_provider, CoauthUpstreamLink, CoauthUpstreamProvider,
};
pub use viewer::{get_viewer, CoauthViewer};
