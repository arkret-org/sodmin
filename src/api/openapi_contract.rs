//! Build-time OpenAPI contract manifest for sodmin's upstream clients.
//!
//! `build.rs` consumes local snapshots under `target/openapi/` when
//! available and validates that the coauth/soland operations used by the
//! typed wrapper modules still exist. Without local services it emits a
//! fallback manifest so regular `cargo check` does not require a running
//! stack; set `SODMIN_OPENAPI_STRICT=1` to make snapshots mandatory.

include!(concat!(env!("OUT_DIR"), "/sodmin_openapi_contracts.rs"));

pub fn coauth_has_operation(method: &str, path: &str) -> bool {
    has_operation(COAUTH_OPENAPI_OPERATIONS, method, path)
}

pub fn soland_has_operation(method: &str, path: &str) -> bool {
    has_operation(SOLAND_OPENAPI_OPERATIONS, method, path)
}

fn has_operation(operations: &[(&str, &str)], method: &str, path: &str) -> bool {
    let method = method.to_ascii_uppercase();
    let path = normalize_path(path);
    operations
        .iter()
        .any(|(m, p)| m.eq_ignore_ascii_case(&method) && normalize_path(p) == path)
}

fn normalize_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut in_param = false;
    for ch in path.chars() {
        match ch {
            '{' => {
                in_param = true;
                out.push_str("{}");
            }
            '}' => in_param = false,
            _ if !in_param => out.push(ch),
            _ => {}
        }
    }
    out
}

pub mod coauth {
    pub const VIEWER: &str = "/_cokret/self/viewer";
    pub const AUDIT_FEED: &str = "/_soland/admin/audit-feed";
    pub const OAUTH2_SESSIONS: &str = "/_soland/admin/oauth2-sessions";
    pub const PERSONAL_SESSIONS: &str = "/_soland/admin/personal-sessions";
    pub const UPSTREAM_OAUTH_PROVIDERS: &str = "/_soland/admin/upstream-oauth-providers";
    pub const UPSTREAM_OAUTH_LINKS: &str = "/_soland/admin/upstream-oauth-links";
    pub const USER_REGISTRATION_TOKENS: &str = "/_soland/admin/user-registration-tokens";
    pub const CONNECTOR_HEALTH: &str = "/_soland/admin/connector-health";
    pub const NOTIFICATION_CHANNELS: &str = "/_soland/admin/notification-channels";
    pub const NOTIFICATION_TEMPLATES: &str = "/_soland/admin/notification-templates";
    pub const NOTIFICATION_TEMPLATES_PUBLISH: &str = "/_soland/admin/notification-templates/publish";
    pub const ACCOUNTS: &str = "/_soland/admin/accounts";
    pub const BRIDGE_DESCRIBE: &str = "/_soland/admin/bridge/describe";
    pub const INTEGRATION_DESCRIBE: &str = "/_cokret/self/integration/describe";
}

pub mod soland {
    pub const SERVER_INFO: &str = "/_soland/admin/server/info";
    pub const SERVER_DESCRIBE: &str = "/_cokret/describe";
    pub const SERVER_STATUS: &str = "/_soland/admin/server/status";
    pub const SERVER_STATS: &str = "/_soland/admin/server/stats";
    pub const SPACES: &str = "/_soland/admin/spaces";
    pub const MODERATION_REPORTS: &str = "/_soland/admin/moderation/reports";
    pub const MODERATION_APPEALS: &str = "/_soland/admin/moderation/appeals";
    pub const FEDERATION_STATUS: &str = "/_soland/admin/federation/status";
    pub const AUTHZ_CAPABILITIES: &str = "/_soland/admin/authz/capabilities";
}

#[cfg(test)]
mod tests {
    use super::{coauth, coauth_has_operation, soland, soland_has_operation};

    #[test]
    fn generated_manifest_covers_coauth_wrapper_roots() {
        assert!(coauth_has_operation("GET", coauth::VIEWER));
        assert!(coauth_has_operation("GET", coauth::ACCOUNTS));
        assert!(coauth_has_operation(
            "POST",
            "/_soland/admin/accounts/{id}/risk-action"
        ));
        assert!(coauth_has_operation(
            "POST",
            "/_soland/admin/accounts/{account_id}/risk-action/{proposal_id}/execute"
        ));
    }

    #[test]
    fn generated_manifest_covers_soland_wrapper_roots() {
        assert!(soland_has_operation("GET", soland::SERVER_INFO));
        assert!(soland_has_operation("GET", soland::SPACES));
        assert!(soland_has_operation(
            "POST",
            "/_soland/admin/realms/{realm_id}/destroy"
        ));
    }
}
