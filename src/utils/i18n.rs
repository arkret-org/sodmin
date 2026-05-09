use std::collections::HashMap;

use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    En,
    ZhCn,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::ZhCn => "zh-CN",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::ZhCn => "\u{4e2d}\u{6587}",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s {
            "en" => Some(Language::En),
            "zh-CN" | "zh_CN" | "zh" => Some(Language::ZhCn),
            _ => None,
        }
    }

    pub fn all() -> &'static [Language] {
        &[Language::En, Language::ZhCn]
    }
}

pub struct I18n {
    translations: HashMap<Language, HashMap<String, String>>,
}

impl I18n {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        translations.insert(Language::En, Self::load_en());
        translations.insert(Language::ZhCn, Self::load_zh_cn());
        Self { translations }
    }

    pub fn t(&self, key: &str, lang: Language) -> String {
        self.translations
            .get(&lang)
            .and_then(|m| m.get(key).cloned())
            .or_else(|| {
                self.translations
                    .get(&Language::En)
                    .and_then(|m| m.get(key).cloned())
            })
            .unwrap_or_else(|| key.to_string())
    }

    pub fn t_with(&self, key: &str, lang: Language, params: &[(&str, &str)]) -> String {
        let mut result = self.t(key, lang);
        for (k, v) in params {
            result = result.replace(&format!("{{{k}}}"), v);
        }
        result
    }

    fn load_en() -> HashMap<String, String> {
        let mut m = HashMap::new();

        // Auth
        m.insert("auth.welcome".into(), "Welcome to {name}".into());
        m.insert("auth.base_url".into(), "Homeserver URL".into());
        m.insert("auth.credentials".into(), "Credentials".into());
        m.insert("auth.access_token".into(), "Access Token".into());
        m.insert("auth.username".into(), "Username".into());
        m.insert("auth.password".into(), "Password".into());
        m.insert("auth.sign_in".into(), "Sign In".into());
        m.insert("auth.sign_out".into(), "Sign Out".into());
        m.insert("auth.sso_sign_in".into(), "Sign in with SSO".into());
        m.insert("auth.server_version".into(), "Server version:".into());
        m.insert("auth.supports_specs".into(), "Supports specs:".into());
        m.insert(
            "auth.oauth_hint".into(),
            "Sign in with your account to access the admin dashboard.".into(),
        );
        m.insert("auth.processing".into(), "Signing in...".into());
        m.insert("auth.try_again".into(), "Try again".into());
        m.insert("auth.url_error".into(), "Not a valid homeserver URL".into());

        // Navigation
        m.insert("nav.dashboard".into(), "Dashboard".into());
        m.insert("nav.users".into(), "Users".into());
        m.insert("nav.rooms".into(), "Rooms".into());
        m.insert("nav.media".into(), "Media".into());
        m.insert("nav.reports".into(), "Reports".into());
        m.insert("nav.federation".into(), "Federation".into());
        m.insert(
            "nav.registration_tokens".into(),
            "Registration Tokens".into(),
        );
        m.insert("nav.server_status".into(), "Server Status".into());
        m.insert("nav.server_actions".into(), "Server Actions".into());
        m.insert("nav.notifications".into(), "Notifications".into());
        m.insert("nav.billing".into(), "Billing".into());
        m.insert("nav.server_notices".into(), "Server Notices".into());
        m.insert("nav.management".into(), "Management".into());
        m.insert("nav.section_identity".into(), "Identity".into());
        m.insert("nav.section_moderation".into(), "Moderation".into());
        m.insert("nav.section_infrastructure".into(), "Infrastructure".into());
        m.insert("nav.section_server_ops".into(), "Server Ops".into());
        m.insert("nav.section_pasion".into(), "Identity Provider".into());
        m.insert("nav.audit_log".into(), "Audit Log".into());
        m.insert("nav.oauth2_sessions".into(), "OAuth2 Sessions".into());
        m.insert("nav.personal_tokens".into(), "Personal Tokens".into());
        m.insert("nav.upstream_providers".into(), "Upstream Providers".into());
        m.insert("nav.upstream_links".into(), "Upstream Links".into());
        m.insert("nav.connector_health".into(), "Connector Health".into());
        m.insert(
            "nav.notification_templates".into(),
            "Notification Templates".into(),
        );
        m.insert(
            "nav.notification_channels".into(),
            "Notification Channels".into(),
        );
        m.insert("nav.auth_status".into(), "Auth Status".into());
        m.insert("nav.appservices".into(), "Appservices".into());
        m.insert("nav.logout".into(), "Logout".into());

        // Auth Status page
        m.insert("auth_status.title".into(), "Authentication Status".into());
        m.insert("auth_status.server_info".into(), "Server Connection".into());
        m.insert("auth_status.base_url".into(), "Base URL".into());
        m.insert("auth_status.server_version".into(), "Server Version".into());
        m.insert("auth_status.login_flows".into(), "Login Flows".into());
        m.insert("nav.palpo_admin".into(), "Palpo Admin".into());
        m.insert("nav.server".into(), "Server".into());

        // Header
        m.insert("header.switch_light".into(), "Switch to light mode".into());
        m.insert("header.switch_dark".into(), "Switch to dark mode".into());
        m.insert("header.notifications".into(), "Notifications".into());
        m.insert("header.no_notifications".into(), "No notifications".into());
        m.insert("header.mark_all_read".into(), "Mark all as read".into());
        m.insert("header.clear_all".into(), "Clear all".into());
        m.insert("header.close".into(), "Close".into());

        // Dashboard descriptions
        m.insert("dashboard.server_online".into(), "Server Online".into());
        m.insert("dashboard.pending_reports".into(), "Pending reports".into());
        m.insert(
            "dashboard.avg_api_response".into(),
            "Avg API response time".into(),
        );

        // Auth extra
        m.insert(
            "auth.sign_in_subtitle".into(),
            "Sign in to manage your server".into(),
        );
        m.insert(
            "auth.footer".into(),
            "Palpo Admin - Matrix Server Management".into(),
        );
        m.insert(
            "auth.not_admin".into(),
            "This account is not a server administrator".into(),
        );

        // Users
        m.insert("users.title".into(), "Users".into());
        m.insert("users.subtitle".into(), "Manage Matrix users".into());
        m.insert("users.create".into(), "Create User".into());
        m.insert("users.user_id".into(), "User ID".into());
        m.insert("users.display_name".into(), "Display Name".into());
        m.insert("users.admin".into(), "Admin".into());
        m.insert("users.status".into(), "Status".into());
        m.insert("users.active".into(), "Active".into());
        m.insert("users.deactivated".into(), "Deactivated".into());
        m.insert("users.guest".into(), "Guest".into());
        m.insert("users.created".into(), "Created".into());
        m.insert("users.actions".into(), "Actions".into());
        m.insert("users.view_details".into(), "View Details".into());
        m.insert("users.deactivate".into(), "Deactivate".into());
        m.insert("users.reactivate".into(), "Reactivate".into());
        m.insert("users.delete".into(), "Delete".into());
        m.insert("users.suspend".into(), "Suspend".into());
        m.insert("users.erase".into(), "Erase (GDPR)".into());
        m.insert("users.send_notice".into(), "Send Server Notice".into());
        m.insert("users.search".into(), "Search users...".into());
        m.insert("users.edit".into(), "Edit User".into());
        m.insert("users.reset_password".into(), "Reset Password".into());
        m.insert("users.shadow_ban".into(), "Shadow Ban".into());
        m.insert("users.locked".into(), "Locked".into());
        m.insert("users.suspended".into(), "Suspended".into());
        m.insert("users.no_users".into(), "No users found".into());
        m.insert("users.export_csv".into(), "Export CSV".into());
        m.insert("users.import_csv".into(), "Import CSV".into());
        m.insert("users.create_new".into(), "Create New User".into());
        m.insert("users.password_confirm".into(), "Confirm Password".into());
        m.insert("users.admin_privileges".into(), "Admin Privileges".into());
        m.insert("users.user_type".into(), "User Type".into());
        m.insert("users.regular".into(), "Regular".into());
        m.insert("users.bot".into(), "Bot".into());
        m.insert("users.support".into(), "Support".into());
        m.insert("users.username".into(), "Username".into());
        m.insert("users.password".into(), "Password".into());
        m.insert("users.email_optional".into(), "Email (optional)".into());
        m.insert("users.phone_optional".into(), "Phone (optional)".into());
        m.insert("users.set_admin".into(), "Set Admin".into());
        m.insert("users.exporting".into(), "Exporting...".into());
        m.insert("users.import_users".into(), "Import Users from CSV".into());
        m.insert("users.cancel_edit".into(), "Cancel Edit".into());
        m.insert("users.save_changes".into(), "Save Changes".into());
        m.insert("users.user_info".into(), "User Information".into());
        m.insert("users.account_details".into(), "Account Details".into());
        m.insert("users.third_party_ids".into(), "Third-party IDs".into());
        m.insert("users.overview".into(), "Overview".into());
        m.insert("users.rooms".into(), "Rooms".into());
        m.insert("users.devices".into(), "Devices".into());
        m.insert("users.sessions".into(), "Sessions".into());
        m.insert("users.account_data".into(), "Account Data".into());
        m.insert("users.rate_limits".into(), "Rate Limits".into());
        m.insert("users.features".into(), "Features".into());
        m.insert("users.joined_rooms".into(), "Joined Rooms".into());
        m.insert("users.shadow_banned".into(), "Shadow Banned".into());
        m.insert("users.deactivate_user".into(), "Deactivate User".into());
        m.insert("users.new_password".into(), "New Password".into());
        m.insert("users.no_devices".into(), "No devices found.".into());
        m.insert("users.remove_threepid".into(), "Remove".into());
        m.insert("users.verified".into(), "Verified".into());
        m.insert("users.unverified".into(), "Unverified".into());
        m.insert("common.remove".into(), "Remove".into());
        m.insert("common.enable".into(), "Enable".into());
        m.insert("common.user".into(), "User".into());
        m.insert("common.rows".into(), "Rows:".into());

        // Rooms
        m.insert("rooms.title".into(), "Rooms".into());
        m.insert("rooms.name".into(), "Name".into());
        m.insert("rooms.alias".into(), "Alias".into());
        m.insert("rooms.members".into(), "Members".into());
        m.insert("rooms.visibility".into(), "Visibility".into());
        m.insert("rooms.public".into(), "Public".into());
        m.insert("rooms.private".into(), "Private".into());
        m.insert("rooms.join_rules".into(), "Join Rules".into());
        m.insert("rooms.encrypted".into(), "Encrypted".into());
        m.insert("rooms.topic".into(), "Topic".into());
        m.insert("rooms.delete".into(), "Delete Room".into());
        m.insert("rooms.block".into(), "Block Room".into());
        m.insert("rooms.unblock".into(), "Unblock Room".into());
        m.insert("rooms.purge_history".into(), "Purge History".into());
        m.insert("rooms.create".into(), "Create Room".into());
        m.insert("rooms.search".into(), "Search rooms...".into());
        m.insert("rooms.no_rooms".into(), "No rooms found".into());
        m.insert("rooms.messages".into(), "Messages".into());
        m.insert("rooms.state_events".into(), "State Events".into());
        m.insert("rooms.hierarchy".into(), "Hierarchy".into());
        m.insert("rooms.aliases".into(), "Aliases".into());
        m.insert("rooms.subtitle".into(), "Manage Matrix rooms".into());
        m.insert("rooms.sort".into(), "Sort:".into());
        m.insert("rooms.all".into(), "All".into());
        m.insert("rooms.delete_selected".into(), "Delete Selected".into());
        m.insert("rooms.block_selected".into(), "Block Selected".into());
        m.insert("rooms.overview".into(), "Overview".into());
        m.insert("rooms.room_information".into(), "Space Information".into());
        m.insert("rooms.room_settings".into(), "Space Settings".into());
        m.insert("rooms.room_id".into(), "Space ID".into());
        m.insert("rooms.canonical_alias".into(), "Canonical Alias".into());
        m.insert("rooms.room_type".into(), "Space Type".into());
        m.insert("rooms.space".into(), "Space".into());
        m.insert("rooms.blocked".into(), "Blocked".into());
        m.insert("rooms.other_aliases".into(), "Other Aliases".into());
        m.insert("rooms.no_members".into(), "No members found.".into());
        m.insert("rooms.no_messages".into(), "No messages found.".into());
        m.insert("rooms.no_state_events".into(), "No state events.".into());
        m.insert("rooms.space_hierarchy".into(), "Space Hierarchy".into());
        m.insert("rooms.no_children".into(), "No child spaces found.".into());
        m.insert("rooms.room_name".into(), "Space Name *".into());
        m.insert("rooms.create_button".into(), "Create".into());
        m.insert("rooms.purge_title".into(), "Purge Room History".into());
        m.insert("rooms.purge_before".into(), "Delete messages before".into());
        m.insert("rooms.created".into(), "Created".into());
        m.insert("rooms.kick".into(), "Kick".into());
        m.insert("rooms.unban".into(), "Unban".into());
        m.insert("rooms.invite".into(), "Invite".into());
        m.insert("rooms.promote".into(), "Make Admin".into());
        m.insert("rooms.invite_user_id".into(), "User ID to invite".into());
        m.insert("rooms.edit_room".into(), "Edit Room".into());
        m.insert("rooms.save_changes".into(), "Save Changes".into());
        m.insert("rooms.add_alias".into(), "Add Alias".into());
        m.insert("rooms.delete_alias".into(), "Delete".into());
        m.insert("rooms.new_alias".into(), "New alias".into());
        m.insert("rooms.forward_extremities_desc".into(), "Forward extremities are the leaf events in the room DAG. Multiple extremities may indicate fragmentation.".into());
        m.insert("rooms.forward_extremities_warning".into(), "Warning: Multiple forward extremities detected. This may indicate DAG fragmentation and could impact performance.".into());
        m.insert("rooms.count".into(), "Count".into());
        m.insert("rooms.directory_listing".into(), "Directory Listing".into());
        m.insert("rooms.published".into(), "Published".into());
        m.insert("rooms.unpublished".into(), "Unpublished".into());
        m.insert("rooms.publish".into(), "Publish".into());
        m.insert("rooms.unpublish".into(), "Unpublish".into());
        m.insert("rooms.event_lookup".into(), "Event Lookup".into());
        m.insert("rooms.lookup".into(), "Lookup".into());

        // Reports
        m.insert("reports.title".into(), "Reports".into());
        m.insert("reports.reporter".into(), "Reporter".into());
        m.insert("reports.reason".into(), "Reason".into());
        m.insert("reports.received".into(), "Received".into());
        m.insert("reports.status".into(), "Status".into());
        m.insert("reports.new".into(), "New".into());
        m.insert("reports.in_review".into(), "In Review".into());
        m.insert("reports.resolved".into(), "Resolved".into());
        m.insert("reports.redact".into(), "Redact Event".into());
        m.insert("reports.ban_user".into(), "Ban User".into());
        m.insert("reports.block_room".into(), "Block Space".into());
        m.insert("reports.delete_report".into(), "Delete Report".into());
        m.insert("reports.no_reports".into(), "No reports".into());
        m.insert(
            "reports.subtitle".into(),
            "Event reports submitted by users".into(),
        );
        m.insert("reports.id".into(), "ID".into());
        m.insert("reports.room".into(), "Space".into());
        m.insert("reports.report_details".into(), "Report Details".into());
        m.insert("reports.reporter_user_id".into(), "Reporter Actor ID".into());
        m.insert("reports.room_id".into(), "Space ID".into());
        m.insert("reports.event_id".into(), "Event ID".into());
        m.insert("reports.sender".into(), "Sender".into());
        m.insert("reports.score".into(), "Score".into());

        // Media
        m.insert("media.title".into(), "Media".into());
        m.insert("media.user_id".into(), "User ID".into());
        m.insert("media.media_count".into(), "Media Count".into());
        m.insert("media.total_size".into(), "Total Size".into());
        m.insert("media.actions".into(), "Actions".into());
        m.insert("media.quarantine".into(), "Quarantine".into());
        m.insert("media.delete_local".into(), "Delete Local Media".into());
        m.insert("media.purge_remote".into(), "Purge Remote Media".into());
        m.insert("media.search".into(), "Search users...".into());
        m.insert(
            "media.subtitle".into(),
            "Media usage statistics by user".into(),
        );
        m.insert("media.display_name".into(), "Display Name".into());
        m.insert("media.no_media".into(), "No media statistics found".into());

        // Server
        m.insert("server.version".into(), "Server Version".into());
        m.insert("server.features".into(), "Server Features".into());
        m.insert("server.status".into(), "Server Status".into());
        m.insert("server.online".into(), "Online".into());
        m.insert("server.healthy".into(), "Healthy".into());
        m.insert("server.issues_detected".into(), "Issues Detected".into());
        m.insert("server.maintenance_mode".into(), "Maintenance Mode".into());
        m.insert("server.actions".into(), "Server Actions".into());
        m.insert("server.notifications".into(), "Notifications".into());
        m.insert("server.clear_all".into(), "Clear All".into());
        m.insert("server.no_notifications".into(), "No notifications".into());

        // Common
        m.insert("common.save".into(), "Save".into());
        m.insert("common.cancel".into(), "Cancel".into());
        m.insert("common.delete".into(), "Delete".into());
        m.insert("common.confirm".into(), "Confirm".into());
        m.insert("common.search".into(), "Search...".into());
        m.insert("common.loading".into(), "Loading...".into());
        m.insert("common.error".into(), "Error".into());
        m.insert("common.success".into(), "Success".into());
        m.insert("common.no_results".into(), "No results found".into());
        m.insert("common.yes".into(), "Yes".into());
        m.insert("common.no".into(), "No".into());
        m.insert("common.previous".into(), "Previous".into());
        m.insert("common.next".into(), "Next".into());
        m.insert("common.retry".into(), "Retry".into());
        m.insert("common.close".into(), "Close".into());
        m.insert("common.edit".into(), "Edit".into());
        m.insert("common.view".into(), "View".into());
        m.insert("common.selected".into(), "Selected".into());
        m.insert("common.clear_selection".into(), "Clear Selection".into());
        m.insert("common.columns".into(), "Columns".into());
        m.insert("common.refresh".into(), "Refresh".into());
        m.insert("common.load_more".into(), "Load More".into());
        m.insert("common.show".into(), "Show".into());
        m.insert("common.hide".into(), "Hide".into());
        m.insert("common.dismiss".into(), "Dismiss".into());
        m.insert("common.copy".into(), "Copy".into());
        m.insert("common.create".into(), "Create".into());
        m.insert("common.status".into(), "Status".into());
        m.insert("common.name".into(), "Name".into());
        m.insert("common.actions".into(), "Actions".into());
        m.insert("common.failed".into(), "Failed".into());

        // Pasion shared
        m.insert("pasion.status_active".into(), "Active".into());
        m.insert("pasion.status_revoked".into(), "Revoked".into());
        m.insert("pasion.status_healthy".into(), "Healthy".into());
        m.insert("pasion.status_degraded".into(), "Degraded".into());
        m.insert("pasion.status_unhealthy".into(), "Unhealthy".into());
        m.insert("pasion.status_down".into(), "Down".into());

        // Pasion audit log
        m.insert("pasion.audit_log.title".into(), "Audit Log".into());
        m.insert("pasion.audit_log.col_timestamp".into(), "Timestamp".into());
        m.insert("pasion.audit_log.col_operation".into(), "Operation".into());
        m.insert("pasion.audit_log.col_admin".into(), "Admin".into());
        m.insert("pasion.audit_log.col_resource".into(), "Resource".into());
        m.insert("pasion.audit_log.col_ip".into(), "IP Address".into());
        m.insert("pasion.audit_log.col_detail".into(), "Detail".into());

        // Pasion connector health

        // Pasion personal sessions
        m.insert("pasion.personal_sessions.revoke".into(), "Revoke".into());
        m.insert("pasion.personal_sessions.col_scope".into(), "Scope".into());
        m.insert("pasion.personal_sessions.col_owner".into(), "Owner".into());

        // Pasion OAuth2 sessions
        m.insert(
            "pasion.oauth2_sessions.title".into(),
            "OAuth2 Sessions".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.description".into(),
            "Browser and app OAuth2 sessions issued by Pasion".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.empty".into(),
            "No OAuth2 sessions found".into(),
        );
        m.insert("pasion.oauth2_sessions.finish".into(), "Finish".into());
        m.insert(
            "pasion.oauth2_sessions.finish_title".into(),
            "Finish Session".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.finish_description".into(),
            "End this OAuth2 session? The user will be signed out from the corresponding client."
                .into(),
        );
        m.insert(
            "pasion.oauth2_sessions.finished_success".into(),
            "Session finished".into(),
        );

        // Pasion upstream providers

        // Pasion upstream links

        // Pasion notification channels

        // Pasion notification templates

        // Dashboard
        m.insert("dashboard.title".into(), "Dashboard".into());
        m.insert("dashboard.welcome".into(), "Welcome to Palpo Admin".into());
        m.insert("dashboard.total_users".into(), "Total Users".into());
        m.insert("dashboard.total_rooms".into(), "Total Rooms".into());
        m.insert("dashboard.total_reports".into(), "Pending Reports".into());
        m.insert("dashboard.active_users".into(), "Active Users".into());
        m.insert("dashboard.api_latency".into(), "API Latency".into());

        // Registration tokens
        m.insert("registration_tokens.token".into(), "Token".into());
        m.insert("registration_tokens.pending".into(), "Pending".into());
        m.insert("registration_tokens.completed".into(), "Completed".into());
        m.insert("registration_tokens.expiry".into(), "Expiry".into());
        m.insert("registration_tokens.unlimited".into(), "Unlimited".into());
        m.insert("registration_tokens.never".into(), "Never".into());
        m.insert("registration_tokens.create".into(), "Create Token".into());
        m.insert("registration_tokens.actions".into(), "Actions".into());
        m.insert("registration_tokens.delete".into(), "Delete".into());
        m.insert("registration_tokens.filter_all".into(), "All".into());
        m.insert("registration_tokens.filter_active".into(), "Active".into());

        // Destinations
        m.insert("destinations.title".into(), "Federation".into());
        m.insert("destinations.destination".into(), "Destination".into());
        m.insert("destinations.last_failure".into(), "Last Failure".into());
        m.insert("destinations.reset".into(), "Reset Connection".into());
        m.insert("destinations.status".into(), "Status".into());
        m.insert("destinations.last_retry".into(), "Last Retry".into());
        m.insert("destinations.actions".into(), "Actions".into());
        m.insert("destinations.reset_button".into(), "Reset".into());
        m.insert("destinations.failed".into(), "Failed".into());
        m.insert("destinations.ok".into(), "OK".into());
        m.insert("destinations.retry_info".into(), "Retry Info".into());
        m.insert("destinations.status".into(), "Status".into());

        // Billing
        m.insert("billing.title".into(), "Billing".into());
        m.insert("billing.subscription".into(), "Subscription".into());
        m.insert("billing.payment_history".into(), "Payment History".into());
        m.insert("billing.amount".into(), "Amount".into());
        m.insert("billing.date".into(), "Date".into());
        m.insert("billing.invoice".into(), "Invoice".into());

        // Server Notices
        m.insert("server_notices.title".into(), "Server Notices".into());
        m.insert("server_notices.single_user".into(), "Single User".into());
        m.insert("server_notices.broadcast".into(), "Broadcast to All".into());
        m.insert("server_notices.user_id".into(), "User ID".into());
        m.insert("server_notices.message".into(), "Message".into());
        m.insert("server_notices.send_notice".into(), "Send Notice".into());
        m.insert("server_notices.user".into(), "User".into());
        m.insert("server_notices.timestamp".into(), "Timestamp".into());
        m.insert("server_notices.event_id".into(), "Event ID".into());

        // Not Found
        m.insert("not_found.title".into(), "404".into());
        m.insert("not_found.message".into(), "Page not found".into());
        m.insert("not_found.go_dashboard".into(), "Go to Dashboard".into());

        // Language
        m.insert("language.en".into(), "English".into());
        m.insert("language.zh_cn".into(), "\u{4e2d}\u{6587}".into());
        m.insert("language.select".into(), "Language".into());

        apply_contrix_overrides(&mut m, Language::En);
        m
    }

    fn load_zh_cn() -> HashMap<String, String> {
        let mut m = HashMap::new();

        // Auth
        m.insert("auth.username".into(), "\u{7528}\u{6237}\u{540d}".into());
        m.insert("auth.password".into(), "\u{5bc6}\u{7801}".into());
        m.insert("auth.sign_in".into(), "\u{767b}\u{5f55}".into());
        m.insert("auth.sign_out".into(), "\u{9000}\u{51fa}".into());
        m.insert("auth.rate_limited".into(), "\u{5c1d}\u{8bd5}\u{6b21}\u{6570}\u{8fc7}\u{591a}\u{ff0c}\u{8bf7}\u{7a0d}\u{540e}\u{518d}\u{8bd5}".into());
        m.insert("auth.oauth_hint".into(), "\u{767b}\u{5f55}\u{60a8}\u{7684}\u{8d26}\u{6237}\u{4ee5}\u{8bbf}\u{95ee}\u{7ba1}\u{7406}\u{540e}\u{53f0}".into());
        m.insert("auth.try_again".into(), "\u{91cd}\u{8bd5}".into());

        // Navigation
        m.insert("nav.dashboard".into(), "\u{4eea}\u{8868}\u{76d8}".into());
        m.insert("nav.users".into(), "\u{7528}\u{6237}".into());
        m.insert("nav.rooms".into(), "\u{623f}\u{95f4}".into());
        m.insert("nav.media".into(), "\u{5a92}\u{4f53}".into());
        m.insert("nav.reports".into(), "\u{4e3e}\u{62a5}".into());
        m.insert("nav.federation".into(), "\u{8054}\u{90a6}".into());
        m.insert(
            "nav.registration_tokens".into(),
            "\u{6ce8}\u{518c}\u{4ee4}\u{724c}".into(),
        );
        m.insert(
            "nav.server_status".into(),
            "\u{670d}\u{52a1}\u{5668}\u{72b6}\u{6001}".into(),
        );
        m.insert("nav.notifications".into(), "\u{901a}\u{77e5}".into());
        m.insert("nav.billing".into(), "\u{8d26}\u{5355}".into());
        m.insert("nav.management".into(), "\u{7ba1}\u{7406}".into());
        m.insert("nav.section_identity".into(), "\u{8eab}\u{4efd}".into());
        m.insert("nav.section_moderation".into(), "\u{5ba1}\u{6838}".into());
        m.insert(
            "nav.section_infrastructure".into(),
            "\u{57fa}\u{7840}\u{8bbe}\u{65bd}".into(),
        );
        m.insert(
            "nav.section_server_ops".into(),
            "\u{670d}\u{52a1}\u{5668}\u{64cd}\u{4f5c}".into(),
        );
        m.insert(
            "nav.audit_log".into(),
            "\u{5ba1}\u{8ba1}\u{65e5}\u{5fd7}".into(),
        );
        m.insert(
            "nav.oauth2_sessions".into(),
            "OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "nav.personal_tokens".into(),
            "\u{4e2a}\u{4eba}\u{4ee4}\u{724c}".into(),
        );
        m.insert(
            "nav.upstream_providers".into(),
            "\u{4e0a}\u{6e38}\u{63d0}\u{4f9b}\u{8005}".into(),
        );
        m.insert(
            "nav.upstream_links".into(),
            "\u{4e0a}\u{6e38}\u{94fe}\u{63a5}".into(),
        );
        m.insert(
            "nav.connector_health".into(),
            "\u{8fde}\u{63a5}\u{5668}\u{5065}\u{5eb7}".into(),
        );
        m.insert(
            "nav.notification_templates".into(),
            "\u{901a}\u{77e5}\u{6a21}\u{677f}".into(),
        );
        m.insert(
            "nav.notification_channels".into(),
            "\u{901a}\u{77e5}\u{6e20}\u{9053}".into(),
        );
        // 应用服务
        m.insert("nav.logout".into(), "\u{9000}\u{51fa}".into());

        // Auth Status page
        m.insert("auth_status.subtitle".into(), "\u{59d4}\u{6258}\u{8ba4}\u{8bc1}\u{80fd}\u{529b}\u{548c}\u{767b}\u{5f55}\u{6d41}\u{72b6}\u{6001}".into());
        m.insert("auth_status.base_url".into(), "\u{57fa}\u{7840} URL".into());
        m.insert("auth_status.auth_capabilities_desc".into(), "\u{6b64}\u{670d}\u{52a1}\u{5668}\u{652f}\u{6301}\u{7684}\u{767b}\u{5f55}\u{65b9}\u{5f0f}".into());
        m.insert("auth_status.no_password_warning".into(), "\u{6b64}\u{670d}\u{52a1}\u{5668}\u{4e0d}\u{652f}\u{6301}\u{5bc6}\u{7801}\u{767b}\u{5f55}\u{3002}\u{7528}\u{6237}\u{5fc5}\u{987b}\u{901a}\u{8fc7} SSO \u{6216}\u{8bbf}\u{95ee}\u{4ee4}\u{724c}\u{8fdb}\u{884c}\u{8eab}\u{4efd}\u{9a8c}\u{8bc1}\u{3002}".into());
        m.insert("auth_status.sso_only_hint".into(), "\u{6b64}\u{670d}\u{52a1}\u{5668}\u{914d}\u{7f6e}\u{4e3a}\u{4ec5} SSO \u{8ba4}\u{8bc1}\u{3002}\u{7ba1}\u{7406}\u{5458}\u{8bbf}\u{95ee}\u{9700}\u{8981}\u{8bbf}\u{95ee}\u{4ee4}\u{724c}\u{6216} SSO \u{4f1a}\u{8bdd}\u{3002}".into());
        m.insert("auth_status.diagnostics_desc".into(), "\u{68c0}\u{67e5} OIDC/MAS \u{53d1}\u{884c}\u{8005}\u{53d1}\u{73b0}\u{3001}\u{7aef}\u{70b9}\u{53ef}\u{7528}\u{6027}\u{3001}DCR \u{652f}\u{6301}\u{548c}\u{4f5c}\u{7528}\u{57df}\u{914d}\u{7f6e}".into());
        m.insert("auth_status.dev_diagnostics_desc".into(), "\u{63a2}\u{6d4b} MAS/Pasion \u{7aef}\u{70b9}\u{4ee5}\u{8c03}\u{8bd5}\u{6ce8}\u{518c}\u{3001}\u{540c}\u{610f}\u{548c} well-known \u{53d1}\u{73b0}".into());
        m.insert("nav.palpo_admin".into(), "Palpo Admin".into());
        m.insert("nav.server".into(), "\u{670d}\u{52a1}\u{5668}".into());

        // Header
        m.insert(
            "header.switch_light".into(),
            "\u{5207}\u{6362}\u{5230}\u{6d45}\u{8272}\u{6a21}\u{5f0f}".into(),
        );
        m.insert(
            "header.switch_dark".into(),
            "\u{5207}\u{6362}\u{5230}\u{6df1}\u{8272}\u{6a21}\u{5f0f}".into(),
        );
        m.insert("header.notifications".into(), "\u{901a}\u{77e5}".into());
        m.insert(
            "header.no_notifications".into(),
            "\u{6682}\u{65e0}\u{901a}\u{77e5}".into(),
        );
        m.insert(
            "header.mark_all_read".into(),
            "\u{5168}\u{90e8}\u{6807}\u{8bb0}\u{5df2}\u{8bfb}".into(),
        );
        m.insert(
            "header.clear_all".into(),
            "\u{6e05}\u{9664}\u{5168}\u{90e8}".into(),
        );
        m.insert("header.close".into(), "\u{5173}\u{95ed}".into());

        // Dashboard descriptions
        m.insert(
            "dashboard.server_online".into(),
            "\u{670d}\u{52a1}\u{5668}\u{5728}\u{7ebf}".into(),
        );
        m.insert(
            "dashboard.pending_reports".into(),
            "\u{5f85}\u{5904}\u{7406}\u{4e3e}\u{62a5}".into(),
        );
        m.insert(
            "dashboard.avg_api_response".into(),
            "\u{5e73}\u{5747} API \u{54cd}\u{5e94}\u{65f6}\u{95f4}".into(),
        );
        m.insert("dashboard.spec_description".into(), "\u{652f}\u{6301}\u{7684} Matrix \u{89c4}\u{8303}\u{7248}\u{672c}\u{548c}\u{5b9e}\u{9a8c}\u{6027}\u{529f}\u{80fd}".into());

        // Auth extra
        m.insert(
            "auth.sign_in_subtitle".into(),
            "\u{767b}\u{5f55}\u{4ee5}\u{7ba1}\u{7406}\u{60a8}\u{7684}\u{670d}\u{52a1}\u{5668}"
                .into(),
        );
        m.insert(
            "auth.footer".into(),
            "Palpo Admin - Matrix \u{670d}\u{52a1}\u{5668}\u{7ba1}\u{7406}".into(),
        );
        m.insert(
            "auth.not_admin".into(),
            "\u{8be5}\u{8d26}\u{6237}\u{4e0d}\u{662f}\u{670d}\u{52a1}\u{5668}\u{7ba1}\u{7406}\u{5458}".into(),
        );

        // Users
        m.insert("users.user_id".into(), "\u{7528}\u{6237} ID".into());
        m.insert("users.admin".into(), "\u{7ba1}\u{7406}\u{5458}".into());
        m.insert("users.status".into(), "\u{72b6}\u{6001}".into());
        m.insert("users.active".into(), "\u{6d3b}\u{8dc3}".into());
        m.insert("users.guest".into(), "\u{8bbf}\u{5ba2}".into());
        m.insert("users.actions".into(), "\u{64cd}\u{4f5c}".into());
        m.insert("users.deactivate".into(), "\u{505c}\u{7528}".into());
        m.insert("users.delete".into(), "\u{5220}\u{9664}".into());
        m.insert("users.suspend".into(), "\u{6682}\u{505c}".into());
        m.insert("users.erase".into(), "\u{64e6}\u{9664} (GDPR)".into());
        m.insert("users.locked".into(), "\u{5df2}\u{9501}\u{5b9a}".into());
        m.insert("users.suspended".into(), "\u{5df2}\u{6682}\u{505c}".into());
        m.insert("users.export_csv".into(), "\u{5bfc}\u{51fa} CSV".into());
        m.insert("users.import_csv".into(), "\u{5bfc}\u{5165} CSV".into());
        m.insert("users.regular".into(), "\u{666e}\u{901a}".into());
        m.insert("users.bot".into(), "\u{673a}\u{5668}\u{4eba}".into());
        m.insert("users.support".into(), "\u{5ba2}\u{670d}".into());
        m.insert("users.username".into(), "\u{7528}\u{6237}\u{540d}".into());
        m.insert("users.password".into(), "\u{5bc6}\u{7801}".into());
        m.insert("users.overview".into(), "\u{6982}\u{89c8}".into());
        m.insert("users.rooms".into(), "\u{623f}\u{95f4}".into());
        m.insert("users.devices".into(), "\u{8bbe}\u{5907}".into());
        m.insert("users.sessions".into(), "\u{4f1a}\u{8bdd}".into());
        m.insert("users.features".into(), "\u{529f}\u{80fd}".into());
        m.insert("users.remove_threepid".into(), "\u{79fb}\u{9664}".into());
        m.insert("users.verified".into(), "\u{5df2}\u{9a8c}\u{8bc1}".into());
        m.insert("users.unverified".into(), "\u{672a}\u{9a8c}\u{8bc1}".into());
        m.insert("common.remove".into(), "\u{79fb}\u{9664}".into());
        m.insert("common.enable".into(), "\u{542f}\u{7528}".into());
        m.insert("common.user".into(), "\u{7528}\u{6237}".into());
        m.insert("common.rows".into(), "\u{884c}\u{6570}\u{ff1a}".into());

        // Rooms
        m.insert("rooms.name".into(), "\u{540d}\u{79f0}".into());
        m.insert("rooms.alias".into(), "\u{522b}\u{540d}".into());
        m.insert("rooms.members".into(), "\u{6210}\u{5458}".into());
        m.insert("rooms.visibility".into(), "\u{53ef}\u{89c1}\u{6027}".into());
        m.insert("rooms.public".into(), "\u{516c}\u{5f00}".into());
        m.insert("rooms.private".into(), "\u{79c1}\u{5bc6}".into());
        m.insert("rooms.encrypted".into(), "\u{5df2}\u{52a0}\u{5bc6}".into());
        m.insert("rooms.topic".into(), "\u{8bdd}\u{9898}".into());
        m.insert("rooms.messages".into(), "\u{6d88}\u{606f}".into());
        m.insert("rooms.sort".into(), "\u{6392}\u{5e8f}\u{ff1a}".into());
        m.insert("rooms.all".into(), "\u{5168}\u{90e8}".into());
        m.insert("rooms.no_rooms_description".into(), "\u{6ca1}\u{6709}\u{7b26}\u{5408}\u{641c}\u{7d22}\u{6761}\u{4ef6}\u{7684}\u{623f}\u{95f4}".into());
        m.insert("rooms.overview".into(), "\u{6982}\u{89c8}".into());
        m.insert("rooms.room_id".into(), "Space ID".into());
        m.insert("rooms.space".into(), "\u{7a7a}\u{95f4}".into());
        m.insert("rooms.blocked".into(), "\u{5df2}\u{5c01}\u{9501}".into());
        m.insert("rooms.space_hierarchy_desc".into(), "\u{6b64}\u{7a7a}\u{95f4}\u{4e2d}\u{7684}\u{5b50}\u{623f}\u{95f4}\u{548c}\u{5b50}\u{7a7a}\u{95f4}".into());
        m.insert("rooms.create_button".into(), "\u{521b}\u{5efa}".into());
        m.insert("rooms.purge_description".into(), "\u{5220}\u{9664}\u{6307}\u{5b9a}\u{65e5}\u{671f}\u{4e4b}\u{524d}\u{7684}\u{6240}\u{6709}\u{6d88}\u{606f}\u{3002}\u{6b64}\u{64cd}\u{4f5c}\u{4e0d}\u{53ef}\u{64a4}\u{9500}\u{3002}".into());
        m.insert("rooms.purge_warning".into(), "\u{8b66}\u{544a}\u{ff1a}\u{8fd9}\u{5c06}\u{6c38}\u{4e45}\u{5220}\u{9664}\u{6240}\u{9009}\u{65e5}\u{671f}\u{4e4b}\u{524d}\u{7684}\u{6240}\u{6709}\u{6d88}\u{606f}\u{3002}".into());
        m.insert("rooms.delete_confirm".into(), "\u{786e}\u{5b9a}\u{8981}\u{5220}\u{9664}\u{6b64}\u{623f}\u{95f4}\u{5417}\u{ff1f}\u{6240}\u{6709}\u{6d88}\u{606f}\u{548c}\u{5a92}\u{4f53}\u{5c06}\u{88ab}\u{6c38}\u{4e45}\u{5220}\u{9664}\u{3002}\u{6b64}\u{64cd}\u{4f5c}\u{4e0d}\u{53ef}\u{64a4}\u{9500}\u{3002}".into());
        m.insert("rooms.kick".into(), "\u{8e22}\u{51fa}".into());
        m.insert("rooms.invite".into(), "\u{9080}\u{8bf7}".into());
        m.insert("rooms.delete_alias".into(), "\u{5220}\u{9664}".into());
        m.insert("rooms.new_alias".into(), "\u{65b0}\u{522b}\u{540d}".into());
        m.insert("rooms.forward_extremities".into(), "前向极端事件".into());
        m.insert("rooms.count".into(), "数量".into());
        m.insert("rooms.directory_listing".into(), "目录列表".into());
        m.insert("rooms.published".into(), "已发布".into());
        m.insert("rooms.unpublished".into(), "未发布".into());
        m.insert("rooms.publish".into(), "发布".into());
        m.insert("rooms.unpublish".into(), "取消发布".into());
        m.insert("rooms.event_lookup".into(), "事件查找".into());
        m.insert("rooms.lookup".into(), "查找".into());

        // Reports
        m.insert(
            "reports.title".into(),
            "\u{4e3e}\u{62a5}\u{7ba1}\u{7406}".into(),
        );
        m.insert("reports.reporter".into(), "\u{4e3e}\u{62a5}\u{8005}".into());
        m.insert("reports.reason".into(), "\u{539f}\u{56e0}".into());
        m.insert("reports.status".into(), "\u{72b6}\u{6001}".into());
        m.insert("reports.new".into(), "\u{65b0}\u{4e3e}\u{62a5}".into());
        m.insert("reports.resolved".into(), "\u{5df2}\u{89e3}\u{51b3}".into());
        m.insert(
            "reports.delete_report".into(),
            "\u{5220}\u{9664}\u{4e3e}\u{62a5}".into(),
        );
        m.insert(
            "reports.no_reports".into(),
            "\u{6682}\u{65e0}\u{4e3e}\u{62a5}".into(),
        );
        m.insert(
            "reports.subtitle".into(),
            "\u{7528}\u{6237}\u{63d0}\u{4ea4}\u{7684}\u{4e8b}\u{4ef6}\u{4e3e}\u{62a5}".into(),
        );
        m.insert("reports.id".into(), "ID".into());
        m.insert("reports.room".into(), "Space".into());
        m.insert("reports.no_reports_description".into(), "\u{76ee}\u{524d}\u{6ca1}\u{6709}\u{9700}\u{8981}\u{5ba1}\u{67e5}\u{7684}\u{4e8b}\u{4ef6}\u{4e3e}\u{62a5}".into());
        m.insert(
            "reports.report_details".into(),
            "\u{4e3e}\u{62a5}\u{8be6}\u{60c5}".into(),
        );
        m.insert("reports.room_id".into(), "Space ID".into());
        m.insert("reports.event_id".into(), "\u{4e8b}\u{4ef6} ID".into());
        m.insert("reports.sender".into(), "\u{53d1}\u{9001}\u{8005}".into());
        m.insert("reports.score".into(), "\u{8bc4}\u{5206}".into());

        // Media
        m.insert(
            "media.title".into(),
            "\u{5a92}\u{4f53}\u{7ba1}\u{7406}".into(),
        );
        m.insert("media.user_id".into(), "\u{7528}\u{6237} ID".into());
        m.insert("media.total_size".into(), "\u{603b}\u{5927}\u{5c0f}".into());
        m.insert("media.actions".into(), "\u{64cd}\u{4f5c}".into());
        m.insert("media.quarantine".into(), "\u{9694}\u{79bb}".into());
        m.insert(
            "media.search".into(),
            "\u{641c}\u{7d22}\u{7528}\u{6237}...".into(),
        );
        m.insert("media.subtitle".into(), "\u{6309}\u{7528}\u{6237}\u{7edf}\u{8ba1}\u{5a92}\u{4f53}\u{4f7f}\u{7528}\u{60c5}\u{51b5}".into());
        m.insert(
            "media.display_name".into(),
            "\u{663e}\u{793a}\u{540d}\u{79f0}".into(),
        );
        m.insert(
            "media.no_media".into(),
            "\u{672a}\u{627e}\u{5230}\u{5a92}\u{4f53}\u{7edf}\u{8ba1}\u{6570}\u{636e}".into(),
        );

        // Server
        m.insert(
            "server.version".into(),
            "\u{670d}\u{52a1}\u{5668}\u{7248}\u{672c}".into(),
        );
        m.insert(
            "server.features".into(),
            "\u{670d}\u{52a1}\u{5668}\u{529f}\u{80fd}".into(),
        );
        m.insert("server.online".into(), "\u{5728}\u{7ebf}".into());
        m.insert("server.healthy".into(), "\u{5065}\u{5eb7}".into());
        m.insert("server.notifications".into(), "\u{901a}\u{77e5}".into());

        // Common
        m.insert("common.save".into(), "\u{4fdd}\u{5b58}".into());
        m.insert("common.cancel".into(), "\u{53d6}\u{6d88}".into());
        m.insert("common.delete".into(), "\u{5220}\u{9664}".into());
        m.insert("common.confirm".into(), "\u{786e}\u{8ba4}".into());
        m.insert("common.search".into(), "\u{641c}\u{7d22}...".into());
        m.insert(
            "common.loading".into(),
            "\u{52a0}\u{8f7d}\u{4e2d}...".into(),
        );
        m.insert("common.error".into(), "\u{9519}\u{8bef}".into());
        m.insert("common.success".into(), "\u{6210}\u{529f}".into());
        m.insert("common.yes".into(), "\u{662f}".into());
        m.insert("common.no".into(), "\u{5426}".into());
        m.insert("common.previous".into(), "\u{4e0a}\u{4e00}\u{9875}".into());
        m.insert("common.next".into(), "\u{4e0b}\u{4e00}\u{9875}".into());
        m.insert("common.retry".into(), "\u{91cd}\u{8bd5}".into());
        m.insert("common.close".into(), "\u{5173}\u{95ed}".into());
        m.insert("common.edit".into(), "\u{7f16}\u{8f91}".into());
        m.insert("common.view".into(), "\u{67e5}\u{770b}".into());
        m.insert("common.selected".into(), "\u{5df2}\u{9009}\u{62e9}".into());
        m.insert("common.columns".into(), "\u{5217}".into());
        // 刷新 / 加载更多 / 显示 / 隐藏 / 关闭提示 / 复制 / 创建 / 状态 / 名称 / 操作 / 失败
        m.insert("common.refresh".into(), "\u{5237}\u{65b0}".into());
        m.insert("common.show".into(), "\u{663e}\u{793a}".into());
        m.insert("common.hide".into(), "\u{9690}\u{85cf}".into());
        m.insert("common.dismiss".into(), "\u{5173}\u{95ed}".into());
        m.insert("common.copy".into(), "\u{590d}\u{5236}".into());
        m.insert("common.create".into(), "\u{521b}\u{5efa}".into());
        m.insert("common.status".into(), "\u{72b6}\u{6001}".into());
        m.insert("common.name".into(), "\u{540d}\u{79f0}".into());
        m.insert("common.actions".into(), "\u{64cd}\u{4f5c}".into());
        m.insert("common.failed".into(), "\u{5931}\u{8d25}".into());

        // Pasion 共享 —— 状态标签
        m.insert("pasion.status_active".into(), "\u{6d3b}\u{8dc3}".into());
        m.insert("pasion.status_healthy".into(), "\u{5065}\u{5eb7}".into());
        m.insert("pasion.status_degraded".into(), "\u{964d}\u{7ea7}".into());

        // Pasion 审计日志

        // Pasion 连接器健康

        // Pasion 个人访问令牌

        // Pasion OAuth2 会话
        m.insert(
            "pasion.oauth2_sessions.title".into(),
            "OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.description".into(),
            "Pasion \u{7b7e}\u{53d1}\u{7684}\u{6d4f}\u{89c8}\u{5668}\u{548c}\u{5e94}\u{7528} OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.empty".into(),
            "\u{6682}\u{65e0} OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.finish".into(),
            "\u{7ed3}\u{675f}".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.finish_title".into(),
            "\u{7ed3}\u{675f}\u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.finish_description".into(),
            "\u{786e}\u{8ba4}\u{7ed3}\u{675f}\u{6b64} OAuth2 \u{4f1a}\u{8bdd}\u{5417}\u{ff1f}\u{5bf9}\u{5e94}\u{5ba2}\u{6237}\u{7aef}\u{7684}\u{7528}\u{6237}\u{5c06}\u{88ab}\u{767b}\u{51fa}\u{3002}".into(),
        );
        m.insert(
            "pasion.oauth2_sessions.finished_success".into(),
            "\u{4f1a}\u{8bdd}\u{5df2}\u{7ed3}\u{675f}".into(),
        );

        // Pasion 上游提供者

        // Pasion 上游绑定

        // Pasion 通知渠道

        // Pasion 通知模板

        // Dashboard
        m.insert("dashboard.title".into(), "\u{4eea}\u{8868}\u{76d8}".into());
        m.insert(
            "dashboard.welcome".into(),
            "\u{6b22}\u{8fce}\u{4f7f}\u{7528} Palpo Admin".into(),
        );
        m.insert(
            "dashboard.active_users".into(),
            "\u{6d3b}\u{8dc3}\u{7528}\u{6237}".into(),
        );
        m.insert(
            "dashboard.api_latency".into(),
            "API \u{5ef6}\u{8fdf}".into(),
        );

        // Registration tokens
        m.insert("registration_tokens.no_tokens_description".into(), "\u{521b}\u{5efa}\u{4ee4}\u{724c}\u{4ee5}\u{5141}\u{8bb8}\u{65b0}\u{7528}\u{6237}\u{6ce8}\u{518c}".into());
        m.insert("registration_tokens.create_description".into(), "\u{521b}\u{5efa}\u{5e26}\u{53ef}\u{9009}\u{7ea6}\u{675f}\u{7684}\u{65b0}\u{6ce8}\u{518c}\u{4ee4}\u{724c}".into());

        // Destinations
        m.insert("destinations.status".into(), "\u{72b6}\u{6001}".into());
        m.insert("destinations.actions".into(), "\u{64cd}\u{4f5c}".into());
        m.insert("destinations.failed".into(), "\u{5931}\u{8d25}".into());
        m.insert("destinations.ok".into(), "\u{6b63}\u{5e38}".into());
        m.insert("destinations.detail_description".into(), "\u{8054}\u{90a6}\u{76ee}\u{6807}\u{8be6}\u{60c5}\u{548c}\u{91cd}\u{8bd5}\u{72b6}\u{6001}".into());
        m.insert("destinations.status".into(), "\u{72b6}\u{6001}".into());

        // Billing
        m.insert("billing.title".into(), "\u{8d26}\u{5355}".into());
        m.insert("billing.subscription".into(), "\u{8ba2}\u{9605}".into());
        m.insert("billing.amount".into(), "\u{91d1}\u{989d}".into());
        m.insert("billing.date".into(), "\u{65e5}\u{671f}".into());
        m.insert("billing.invoice".into(), "\u{53d1}\u{7968}".into());

        // Server Notices
        m.insert("server_notices.broadcast_desc".into(), "\u{5411}\u{6240}\u{6709}\u{7528}\u{6237}\u{5e7f}\u{64ad}\u{7ba1}\u{7406}\u{901a}\u{77e5}".into());
        m.insert("server_notices.single_desc".into(), "\u{5411}\u{7279}\u{5b9a}\u{7528}\u{6237}\u{53d1}\u{9001}\u{7ba1}\u{7406}\u{901a}\u{77e5}".into());
        m.insert("server_notices.message".into(), "\u{6d88}\u{606f}".into());
        m.insert("server_notices.history_desc".into(), "\u{4e4b}\u{524d}\u{53d1}\u{9001}\u{7684}\u{670d}\u{52a1}\u{5668}\u{901a}\u{77e5}\u{ff08}\u{672c}\u{5730}\u{5b58}\u{50a8}\u{ff09}".into());
        m.insert("server_notices.user".into(), "\u{7528}\u{6237}".into());

        // Not Found
        m.insert("not_found.title".into(), "404".into());
        m.insert(
            "not_found.message".into(),
            "\u{9875}\u{9762}\u{672a}\u{627e}\u{5230}".into(),
        );
        m.insert(
            "not_found.go_dashboard".into(),
            "\u{8fd4}\u{56de}\u{4eea}\u{8868}\u{76d8}".into(),
        );

        // Language
        m.insert("language.en".into(), "English".into());
        m.insert("language.zh_cn".into(), "\u{4e2d}\u{6587}".into());
        m.insert("language.select".into(), "\u{8bed}\u{8a00}".into());

        apply_contrix_overrides(&mut m, Language::ZhCn);
        m
    }
}

fn apply_contrix_overrides(m: &mut HashMap<String, String>, lang: Language) {
    let entries: &[(&str, &str)] = match lang {
        Language::En => &[
            ("nav.actors", "Actors"),
            ("nav.spaces", "Spaces"),
            ("nav.devices", "Devices"),
            ("nav.capabilities", "Capabilities"),
            ("nav.invite_tokens", "Invite Tokens"),
            ("nav.audit", "Audit"),
            ("nav.applets", "Applets"),
            ("nav.agents", "Agents"),
            ("nav.policy", "Policy"),
            ("nav.coauth_accounts", "Accounts"),
            ("nav.section_coauth", "coauth"),
            ("nav.section_anchor", "Anchor / Lattice"),
            ("nav.anchor_bottom", "Bottom Diagnostics"),
            ("nav.anchor_anchorer", "Anchorer Cell"),
            ("nav.anchor_dag", "Anchor DAG"),
            ("auth.footer", "sodmin - Contrix Administration"),
            ("auth.completing_login", "Completing sign-in..."),
            ("dashboard.welcome", "Welcome to Contrix Admin"),
            (
                "dashboard.total_registered_users",
                "Total registered actors",
            ),
            ("dashboard.active_users", "Active actors"),
            (
                "dashboard.spec_description",
                "Contrix protocol profiles, features and conformance coverage",
            ),
            (
                "users.create_subtitle",
                "Add a new actor to the Principal Server",
            ),
            (
                "rooms.search_placeholder",
                "Search spaces by name or alias...",
            ),
            (
                "rooms.no_rooms_description",
                "There are no spaces matching your search criteria.",
            ),
            (
                "rooms.delete_confirm",
                "Are you sure you want to delete this space? Events, messages and media references may be permanently removed. This action cannot be undone.",
            ),
            (
                "reports.subtitle",
                "Moderation and policy reports submitted by actors",
            ),
            ("media.search", "Search actors..."),
            ("media.subtitle", "Blob/media usage statistics by actor"),
            (
                "auth_status.auth_capabilities_desc",
                "Login methods exposed by coauth and the Principal Server admin profile",
            ),
            (
                "auth_status.dev_diagnostics_desc",
                "Probe coauth endpoints for registration, consent and well-known discovery debugging",
            ),
            ("config.error.title", "Runtime configuration unavailable"),
            (
                "config.error.subtitle",
                "Sign-in cannot proceed until /config.json is reachable.",
            ),
            ("config.error.hint_title", "Deployment checklist"),
            (
                "config.error.hint_serve_path",
                "Serve config.json from the same origin as the admin SPA.",
            ),
            (
                "config.error.hint_required_fields",
                "Ensure coauth_public_url is set to the public coauth origin.",
            ),
            (
                "config.error.hint_check_proxy",
                "Verify the reverse proxy does not strip /config.json.",
            ),
            ("audit.title", "Audit Log"),
            ("audit.subtitle", "Admin actions recorded by the Principal Server"),
            ("audit.id", "ID"),
            ("audit.action", "Action"),
            ("audit.actor_id", "Actor"),
            ("audit.target_type", "Target Type"),
            ("audit.target_id", "Target ID"),
            ("audit.timestamp", "Timestamp"),
            ("audit.source_ip", "Source IP"),
            ("audit.no_entries", "No audit entries"),
            ("audit.filter_action", "Action"),
            ("audit.filter_action_placeholder", "e.g. user.suspend"),
            ("audit.filter_actor", "Actor"),
            ("audit.filter_actor_placeholder", "actor id"),
            ("audit.filter_target_type", "Target Type"),
            ("audit.filter_target_type_placeholder", "e.g. user, space"),
            ("audit.filter_target_id", "Target ID"),
            ("audit.filter_target_id_placeholder", "target id"),
            ("audit.filter_since", "Since"),
            ("audit.filter_until", "Until"),
            ("audit.filter_apply", "Apply"),
            ("audit.filter_reset", "Reset"),
            ("coauth.audit_log.title", "coauth Audit Log"),
            (
                "coauth.audit_log.subtitle",
                "Admin and identity-server operations recorded by coauth",
            ),
            ("coauth.audit_log.id", "ID"),
            ("coauth.audit_log.operation", "Operation"),
            ("coauth.audit_log.actor", "Actor"),
            ("coauth.audit_log.target_type", "Target Type"),
            ("coauth.audit_log.target_id", "Target ID"),
            ("coauth.audit_log.timestamp", "Timestamp"),
            ("coauth.audit_log.source_ip", "Source IP"),
            ("coauth.audit_log.no_entries", "No audit entries"),
            ("coauth.audit_log.filter_operation", "Operation"),
            (
                "coauth.audit_log.filter_operation_placeholder",
                "e.g. account.suspend",
            ),
            ("coauth.audit_log.filter_actor", "Actor"),
            ("coauth.audit_log.filter_actor_placeholder", "actor user id"),
            ("coauth.audit_log.filter_target_type", "Target Type"),
            (
                "coauth.audit_log.filter_target_type_placeholder",
                "e.g. account, session",
            ),
            ("coauth.audit_log.filter_target_id", "Target ID"),
            (
                "coauth.audit_log.filter_target_id_placeholder",
                "target id",
            ),
            ("coauth.audit_log.filter_since", "Since"),
            ("coauth.audit_log.filter_until", "Until"),
            ("coauth.audit_log.filter_apply", "Apply"),
            ("coauth.audit_log.filter_reset", "Reset"),
            ("server_status.title", "Server Status"),
            (
                "server_status.subtitle",
                "Health and capability snapshot for each contrix backend",
            ),
            ("server_status.server_info", "Server"),
            ("server_status.version", "Version"),
            ("server_status.protocol_version", "Protocol"),
            ("server_status.server_name", "Server Name"),
            ("server_status.uptime", "Uptime"),
            ("server_status.unable", "Unable to fetch server status"),
            ("server_status.healthy", "Healthy"),
            ("server_status.issues", "Issues Detected"),
            ("server_status.online", "Online"),
            ("server_status.not_configured", "Not Configured"),
            ("server_status.unreachable", "Unreachable"),
            ("server_status.fetch_failed", "Failed to fetch service describe"),
            ("server_status.services_title", "Services"),
            ("server_status.components_title", "Components"),
            ("server_status.service_admin", "Admin (local)"),
            (
                "server_status.service_admin_hint",
                "Local admin proxy / Principal Server describe",
            ),
            ("server_status.service_coauth", "coauth"),
            (
                "server_status.service_coauth_hint",
                "Identity, OAuth and admin scope provider",
            ),
            (
                "server_status.service_coauth_not_configured",
                "Set coauth_public_url in /config.json to populate this card.",
            ),
            ("server_status.service_soland", "soland"),
            (
                "server_status.service_soland_hint",
                "Spaces, federation and policy backend",
            ),
            (
                "server_status.service_soland_not_configured",
                "soland public URL is not yet wired into /config.json.",
            ),
            ("server_status.service_floria", "floria"),
            (
                "server_status.service_floria_hint",
                "Push gateway and crypto / device backend",
            ),
            (
                "server_status.service_floria_not_configured",
                "floria public URL is not yet wired into /config.json.",
            ),
            ("server_status.service_did", "Service DID"),
            ("server_status.service_type", "Service Type"),
            ("server_status.openapi_version", "OpenAPI"),
            ("server_status.schema_registry", "Schema Registry"),
            ("server_status.event_kind_registry", "Event Kind Registry"),
            ("server_status.profiles_label", "Supported Profiles"),
            ("server_status.features_label", "Supported Features"),
        ],
        Language::ZhCn => &[
            ("nav.actors", "Actor"),
            ("nav.spaces", "Space"),
            ("nav.devices", "设备"),
            ("nav.capabilities", "权限能力"),
            ("nav.invite_tokens", "邀请令牌"),
            ("nav.audit", "审计"),
            ("nav.applets", "Applet"),
            ("nav.agents", "Agent"),
            ("nav.policy", "策略"),
            ("nav.coauth_accounts", "账号"),
            ("nav.section_coauth", "coauth"),
            ("nav.section_anchor", "Anchor / Lattice"),
            ("nav.anchor_bottom", "Bottom 诊断"),
            ("nav.anchor_anchorer", "Anchorer Cell"),
            ("nav.anchor_dag", "Anchor DAG"),
            ("auth.footer", "sodmin - Contrix 管理后台"),
            ("auth.completing_login", "正在完成登录..."),
            ("dashboard.welcome", "欢迎使用 Contrix Admin"),
            ("dashboard.active_users", "活跃 Actor"),
            (
                "dashboard.spec_description",
                "Contrix 协议 profile、功能和 conformance 覆盖",
            ),
            (
                "rooms.delete_confirm",
                "确定要删除此 Space 吗？事件、消息和媒体引用可能被永久删除，此操作无法撤销。",
            ),
            ("reports.subtitle", "Actor 提交的审核和策略报告"),
            ("media.search", "搜索 Actor..."),
            ("media.subtitle", "按 Actor 统计 Blob/media 使用量"),
            (
                "auth_status.auth_capabilities_desc",
                "coauth 和 Principal Server admin profile 暴露的登录方式",
            ),
            (
                "auth_status.dev_diagnostics_desc",
                "探测 coauth 端点以调试注册、同意和 well-known 发现",
            ),
            ("config.error.title", "运行时配置不可用"),
            (
                "config.error.subtitle",
                "在 /config.json 可访问之前无法继续登录。",
            ),
            ("config.error.hint_title", "部署排查清单"),
            (
                "config.error.hint_serve_path",
                "将 config.json 与管理 SPA 部署在同一 origin。",
            ),
            (
                "config.error.hint_required_fields",
                "确保 coauth_public_url 指向公开的 coauth origin。",
            ),
            (
                "config.error.hint_check_proxy",
                "确认反向代理没有屏蔽 /config.json。",
            ),
            ("audit.title", "审计日志"),
            ("audit.subtitle", "Principal Server 记录的管理员操作"),
            ("audit.id", "ID"),
            ("audit.action", "操作"),
            ("audit.actor_id", "Actor"),
            ("audit.target_type", "目标类型"),
            ("audit.target_id", "目标 ID"),
            ("audit.timestamp", "时间"),
            ("audit.source_ip", "来源 IP"),
            ("audit.no_entries", "暂无审计记录"),
            ("audit.filter_action", "操作"),
            ("audit.filter_action_placeholder", "如 user.suspend"),
            ("audit.filter_actor", "Actor"),
            ("audit.filter_actor_placeholder", "actor id"),
            ("audit.filter_target_type", "目标类型"),
            ("audit.filter_target_type_placeholder", "如 user、space"),
            ("audit.filter_target_id", "目标 ID"),
            ("audit.filter_target_id_placeholder", "target id"),
            ("audit.filter_since", "起始时间"),
            ("audit.filter_until", "截止时间"),
            ("audit.filter_apply", "应用"),
            ("audit.filter_reset", "重置"),
            ("coauth.audit_log.title", "coauth 审计日志"),
            (
                "coauth.audit_log.subtitle",
                "coauth 记录的管理与身份服务操作",
            ),
            ("coauth.audit_log.id", "ID"),
            ("coauth.audit_log.operation", "操作"),
            ("coauth.audit_log.actor", "Actor"),
            ("coauth.audit_log.target_type", "目标类型"),
            ("coauth.audit_log.target_id", "目标 ID"),
            ("coauth.audit_log.timestamp", "时间"),
            ("coauth.audit_log.source_ip", "来源 IP"),
            ("coauth.audit_log.no_entries", "暂无审计记录"),
            ("coauth.audit_log.filter_operation", "操作"),
            (
                "coauth.audit_log.filter_operation_placeholder",
                "如 account.suspend",
            ),
            ("coauth.audit_log.filter_actor", "Actor"),
            (
                "coauth.audit_log.filter_actor_placeholder",
                "actor user id",
            ),
            ("coauth.audit_log.filter_target_type", "目标类型"),
            (
                "coauth.audit_log.filter_target_type_placeholder",
                "如 account、session",
            ),
            ("coauth.audit_log.filter_target_id", "目标 ID"),
            (
                "coauth.audit_log.filter_target_id_placeholder",
                "target id",
            ),
            ("coauth.audit_log.filter_since", "起始时间"),
            ("coauth.audit_log.filter_until", "截止时间"),
            ("coauth.audit_log.filter_apply", "应用"),
            ("coauth.audit_log.filter_reset", "重置"),
            ("server_status.title", "服务状态"),
            (
                "server_status.subtitle",
                "各 contrix 后端的健康与能力快照",
            ),
            ("server_status.server_info", "服务器"),
            ("server_status.version", "版本"),
            ("server_status.protocol_version", "协议"),
            ("server_status.server_name", "服务器名称"),
            ("server_status.uptime", "运行时长"),
            ("server_status.unable", "无法获取服务器状态"),
            ("server_status.healthy", "健康"),
            ("server_status.issues", "存在异常"),
            ("server_status.online", "在线"),
            ("server_status.not_configured", "未配置"),
            ("server_status.unreachable", "不可达"),
            ("server_status.fetch_failed", "获取 service describe 失败"),
            ("server_status.services_title", "服务"),
            ("server_status.components_title", "组件"),
            ("server_status.service_admin", "Admin（本地）"),
            (
                "server_status.service_admin_hint",
                "本地 admin 代理 / Principal Server describe",
            ),
            ("server_status.service_coauth", "coauth"),
            (
                "server_status.service_coauth_hint",
                "身份、OAuth 与 admin scope 提供方",
            ),
            (
                "server_status.service_coauth_not_configured",
                "在 /config.json 设置 coauth_public_url 即可填充。",
            ),
            ("server_status.service_soland", "soland"),
            (
                "server_status.service_soland_hint",
                "Space、Federation 与策略后端",
            ),
            (
                "server_status.service_soland_not_configured",
                "/config.json 暂未配置 soland 公网 URL。",
            ),
            ("server_status.service_floria", "floria"),
            (
                "server_status.service_floria_hint",
                "推送网关与设备 / 加密后端",
            ),
            (
                "server_status.service_floria_not_configured",
                "/config.json 暂未配置 floria 公网 URL。",
            ),
            ("server_status.service_did", "Service DID"),
            ("server_status.service_type", "Service 类型"),
            ("server_status.openapi_version", "OpenAPI"),
            ("server_status.schema_registry", "Schema Registry"),
            ("server_status.event_kind_registry", "Event Kind Registry"),
            ("server_status.profiles_label", "支持的 Profile"),
            ("server_status.features_label", "支持的 Feature"),
        ],
    };

    for (key, value) in entries {
        m.insert((*key).to_string(), (*value).to_string());
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

impl std::hash::Hash for Language {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Eq for Language {}

thread_local! {
    static I18N: I18n = I18n::new();
}

static CURRENT_LANG: GlobalSignal<Language> = GlobalSignal::new(|| {
    crate::utils::storage::get_item("language")
        .and_then(|s| Language::from_code(&s))
        .unwrap_or(Language::En)
});

pub fn t(key: &str) -> String {
    let lang = *CURRENT_LANG.read();
    I18N.with(|i18n| i18n.t(key, lang))
}

pub fn t_with(key: &str, params: &[(&str, &str)]) -> String {
    let lang = *CURRENT_LANG.read();
    I18N.with(|i18n| i18n.t_with(key, lang, params))
}

pub fn set_language(lang: Language) {
    crate::utils::storage::set_item("language", lang.code());
    *CURRENT_LANG.write() = lang;
}

pub fn current_language() -> Language {
    *CURRENT_LANG.read()
}
