pub mod coauth {
    pub const VIEWER: &str = "/_cokret/self/account/viewer";
    pub const AUDIT_FEED: &str = "/_coauth/admin/audit-feed";
    pub const OAUTH2_SESSIONS: &str = "/_coauth/admin/oauth-sessions";
    pub const PERSONAL_SESSIONS: &str = "/_coauth/admin/personal-sessions";
    pub const UPSTREAM_OAUTH_PROVIDERS: &str = "/_coauth/admin/upstream-oauth-providers";
    pub const UPSTREAM_OAUTH_LINKS: &str = "/_coauth/admin/upstream-oauth-links";
    pub const USER_REGISTRATION_TOKENS: &str = "/_coauth/admin/user-registration-tokens";
    pub const CONNECTOR_HEALTH: &str = "/_coauth/admin/connector-health";
    pub const NOTIFICATION_CHANNELS: &str = "/_coauth/admin/notification-channels";
    pub const NOTIFICATION_TEMPLATES: &str = "/_coauth/admin/notification-templates";
    pub const ACCOUNTS: &str = "/_coauth/admin/accounts";
    pub const BRIDGE_DESCRIBE: &str = "/_coauth/admin/bridge/describe";
    pub const INTEGRATION_DESCRIBE: &str = "/_soland/self/integration/describe";
}

pub mod soland {
    pub const SERVER_INFO: &str = "/_soland/admin/server/info";
    pub const SERVER_DESCRIBE: &str = "/_cokret/describe";
    pub const SERVER_STATUS: &str = "/_soland/admin/server/status";
    pub const SERVER_STATS: &str = "/_soland/admin/server/stats";
}
