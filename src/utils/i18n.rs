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
        m.insert("nav.section_coauth".into(), "Identity Provider".into());
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
        m.insert("nav.contrix_admin".into(), "Cokret Admin".into());
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
            "sodmin - Cokret Administration".into(),
        );
        m.insert(
            "auth.not_admin".into(),
            "This account is not a server administrator".into(),
        );

        // Users
        m.insert("users.title".into(), "Users".into());
        m.insert("users.subtitle".into(), "Manage Cokret actors".into());
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
        m.insert("reports.report_details".into(), "Report Details".into());
        m.insert(
            "reports.reporter_user_id".into(),
            "Reporter Actor ID".into(),
        );
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
        m.insert("common.approve".into(), "Approve".into());
        m.insert("common.revoke".into(), "Revoke".into());
        m.insert("common.suspend".into(), "Suspend".into());
        m.insert("common.reject".into(), "Reject".into());
        m.insert("common.open".into(), "Open".into());

        // coauth shared
        m.insert("coauth.status_active".into(), "Active".into());
        m.insert("coauth.status_revoked".into(), "Revoked".into());
        m.insert("coauth.status_healthy".into(), "Healthy".into());
        m.insert("coauth.status_degraded".into(), "Degraded".into());
        m.insert("coauth.status_unhealthy".into(), "Unhealthy".into());
        m.insert("coauth.status_down".into(), "Down".into());

        // coauth audit log
        m.insert("coauth.audit_log.title".into(), "Audit Log".into());
        m.insert("coauth.audit_log.col_timestamp".into(), "Timestamp".into());
        m.insert("coauth.audit_log.col_operation".into(), "Operation".into());
        m.insert("coauth.audit_log.col_admin".into(), "Admin".into());
        m.insert("coauth.audit_log.col_resource".into(), "Resource".into());
        m.insert("coauth.audit_log.col_ip".into(), "IP Address".into());
        m.insert("coauth.audit_log.col_detail".into(), "Detail".into());

        // coauth connector health

        // coauth personal sessions
        m.insert("coauth.personal_sessions.revoke".into(), "Revoke".into());
        m.insert("coauth.personal_sessions.col_scope".into(), "Scope".into());
        m.insert("coauth.personal_sessions.col_owner".into(), "Owner".into());

        // coauth OAuth2 sessions
        m.insert(
            "coauth.oauth2_sessions.title".into(),
            "OAuth2 Sessions".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.description".into(),
            "Browser and app OAuth2 sessions issued by coauth".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.empty".into(),
            "No OAuth2 sessions found".into(),
        );
        m.insert("coauth.oauth2_sessions.finish".into(), "Finish".into());
        m.insert(
            "coauth.oauth2_sessions.finish_title".into(),
            "Finish Session".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.finish_description".into(),
            "End this OAuth2 session? The user will be signed out from the corresponding client."
                .into(),
        );
        m.insert(
            "coauth.oauth2_sessions.finished_success".into(),
            "Session finished".into(),
        );

        // coauth upstream providers

        // coauth upstream links

        // coauth notification channels

        // coauth notification templates

        // Dashboard
        m.insert("dashboard.title".into(), "Dashboard".into());
        m.insert(
            "dashboard.welcome".into(),
            "Welcome to Cokret Admin".into(),
        );
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
        m.insert("auth_status.dev_diagnostics_desc".into(), "\u{63a2}\u{6d4b} coauth \u{7aef}\u{70b9}\u{4ee5}\u{8c03}\u{8bd5}\u{6ce8}\u{518c}\u{3001}\u{540c}\u{610f}\u{548c} well-known \u{53d1}\u{73b0}".into());
        m.insert("nav.contrix_admin".into(), "Cokret Admin".into());
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
        m.insert(
            "dashboard.spec_description".into(),
            "\u{652f}\u{6301}\u{7684} Cokret profile \u{548c} conformance \u{8986}\u{76d6}".into(),
        );

        // Auth extra
        m.insert(
            "auth.sign_in_subtitle".into(),
            "\u{767b}\u{5f55}\u{4ee5}\u{7ba1}\u{7406}\u{60a8}\u{7684}\u{670d}\u{52a1}\u{5668}"
                .into(),
        );
        m.insert(
            "auth.footer".into(),
            "sodmin - Cokret \u{7ba1}\u{7406}\u{540e}\u{53f0}".into(),
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
        m.insert("common.approve".into(), "\u{6279}\u{51c6}".into());
        m.insert("common.revoke".into(), "\u{64a4}\u{9500}".into());
        // 暂停 — Suspend
        m.insert("common.suspend".into(), "\u{6682}\u{505c}".into());
        // 驳回 — Reject
        m.insert("common.reject".into(), "\u{9a73}\u{56de}".into());
        m.insert("common.open".into(), "\u{6253}\u{5f00}".into());

        // coauth 共享 —— 状态标签
        m.insert("coauth.status_active".into(), "\u{6d3b}\u{8dc3}".into());
        m.insert("coauth.status_healthy".into(), "\u{5065}\u{5eb7}".into());
        m.insert("coauth.status_degraded".into(), "\u{964d}\u{7ea7}".into());

        // coauth 审计日志

        // coauth 连接器健康

        // coauth 个人访问令牌

        // coauth OAuth2 会话
        m.insert(
            "coauth.oauth2_sessions.title".into(),
            "OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.description".into(),
            "coauth \u{7b7e}\u{53d1}\u{7684}\u{6d4f}\u{89c8}\u{5668}\u{548c}\u{5e94}\u{7528} OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.empty".into(),
            "\u{6682}\u{65e0} OAuth2 \u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.finish".into(),
            "\u{7ed3}\u{675f}".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.finish_title".into(),
            "\u{7ed3}\u{675f}\u{4f1a}\u{8bdd}".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.finish_description".into(),
            "\u{786e}\u{8ba4}\u{7ed3}\u{675f}\u{6b64} OAuth2 \u{4f1a}\u{8bdd}\u{5417}\u{ff1f}\u{5bf9}\u{5e94}\u{5ba2}\u{6237}\u{7aef}\u{7684}\u{7528}\u{6237}\u{5c06}\u{88ab}\u{767b}\u{51fa}\u{3002}".into(),
        );
        m.insert(
            "coauth.oauth2_sessions.finished_success".into(),
            "\u{4f1a}\u{8bdd}\u{5df2}\u{7ed3}\u{675f}".into(),
        );

        // coauth 上游提供者

        // coauth 上游绑定

        // coauth 通知渠道

        // coauth 通知模板

        // Dashboard
        m.insert("dashboard.title".into(), "\u{4eea}\u{8868}\u{76d8}".into());
        m.insert(
            "dashboard.welcome".into(),
            "\u{6b22}\u{8fce}\u{4f7f}\u{7528} Cokret Admin".into(),
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
            ("nav.consent", "Consent"),
            ("nav.covered_frontier", "Covered Frontier"),
            ("nav.components", "Components"),
            ("nav.signing_keys", "Signing Keys"),
            ("nav.multisig", "Multi-sig"),
            ("signing_keys.title", "Anchorer signing key"),
            (
                "signing_keys.subtitle",
                "AnchorerWorker signing key origin, verification method and rotation controls.",
            ),
            ("signing_keys.origin", "Origin"),
            ("signing_keys.origin.configured", "Configured"),
            ("signing_keys.origin.ephemeral", "Ephemeral"),
            ("signing_keys.verification_method", "Verification method"),
            ("signing_keys.did", "DID"),
            ("signing_keys.kid", "kid"),
            ("signing_keys.algorithm", "Algorithm"),
            ("signing_keys.last_rotated", "Last rotated"),
            ("signing_keys.rotate", "Rotate signing key"),
            (
                "signing_keys.rotate_confirm_title",
                "Rotate AnchorerWorker signing key?",
            ),
            (
                "signing_keys.rotate_confirm_body",
                "This generates a fresh key and rebinds the AnchorerWorker. Existing in-flight Anchors will be re-signed with the new key. Audit-logged. Continue?",
            ),
            (
                "signing_keys.ephemeral_warning",
                "Anchorer is signing with an EPHEMERAL key. This key will be lost on the next worker restart and cannot be rotated in place — redeploy the principal-server with a configured key (PEM / KMS) before promoting to production.",
            ),
            (
                "signing_keys.read_only_note",
                "(read-only — operator must redeploy with a configured key to rotate)",
            ),
            (
                "signing_keys.rotate_disabled_hint",
                "Rotation disabled for this signing-key origin. Configure a stable PEM / KMS-backed signer and redeploy to enable.",
            ),
            (
                "signing_keys.rotate_enabled_hint",
                "Rotating generates a fresh key, swaps the worker's signer atomically, and reports the new verification method id. Audit-logged.",
            ),
            ("multisig.title", "Multi-sig pending"),
            (
                "multisig.subtitle",
                "Anchors awaiting threshold partial signatures from the anchorer cell members.",
            ),
            ("multisig.col_anchor", "Anchor ID"),
            ("multisig.col_threshold", "Threshold"),
            ("multisig.col_collected", "Collected"),
            ("multisig.col_missing", "Missing signers"),
            ("multisig.col_state_root", "State root"),
            ("multisig.col_created", "Created"),
            ("multisig.submit_partial", "Submit my partial signature"),
            (
                "multisig.empty",
                "All Anchors in this Space have reached threshold and assembled.",
            ),
            ("consent.title", "Consent grants"),
            (
                "consent.subtitle",
                "Read-only join-projection of consent cell or-set values",
            ),
            ("consent.filter_holder", "Filter by holder DID"),
            ("consent.holder_did", "Holder DID"),
            ("consent.peer_did", "Peer DID"),
            ("consent.scope", "Scope"),
            ("consent.status", "Status"),
            ("consent.created", "Created"),
            (
                "consent.no_grants",
                "No consent grants match the current filter.",
            ),
            ("components.title", "Component registry"),
            (
                "components.subtitle",
                "Server-wide component_type / cell_family registry with criticality and version drift indicators.",
            ),
            ("components.col_component", "Component"),
            ("components.col_family", "Cell family"),
            ("components.col_criticality", "Criticality"),
            ("components.col_spec", "Spec"),
            ("components.col_impl", "Impl"),
            ("components.col_status", "Status"),
            ("components.col_drift", "Drift"),
            ("components.empty", "Server reported no components."),
            (
                "components.drift_alert",
                "{count} component(s) report version drift between spec and impl.",
            ),
            ("covered_frontier.title", "covered_frontier lag"),
            (
                "covered_frontier.subtitle",
                "Governance frontier vs MLS group epoch — Move acknowledgement lag.",
            ),
            ("covered_frontier.lag", "Lag"),
            ("covered_frontier.mls_epoch", "MLS epoch"),
            (
                "covered_frontier.governance_size",
                "Governance frontier size",
            ),
            ("covered_frontier.covered_size", "Covered frontier size"),
            ("covered_frontier.latest_anchor", "Latest anchor"),
            (
                "covered_frontier.last_covered",
                "Last covered_frontier update",
            ),
            (
                "covered_frontier.warn",
                "Lag is above warn threshold; investigate MLS group health (member offline, KeyPackage stale).",
            ),
            ("auth.footer", "sodmin - Cokret Administration"),
            ("auth.completing_login", "Completing sign-in..."),
            ("dashboard.welcome", "Welcome to Cokret Admin"),
            (
                "dashboard.total_registered_users",
                "Total registered actors",
            ),
            ("dashboard.active_users", "Active actors"),
            (
                "dashboard.spec_description",
                "Cokret protocol profiles, features and conformance coverage",
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
            (
                "audit.subtitle",
                "Admin actions recorded by the Principal Server",
            ),
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
            ("coauth.audit_log.filter_target_id_placeholder", "target id"),
            ("coauth.audit_log.filter_since", "Since"),
            ("coauth.audit_log.filter_until", "Until"),
            ("coauth.audit_log.filter_apply", "Apply"),
            ("coauth.audit_log.filter_reset", "Reset"),
            ("server_status.title", "Server Status"),
            (
                "server_status.subtitle",
                "Health and capability snapshot for each cokret backend",
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
            (
                "server_status.fetch_failed",
                "Failed to fetch service describe",
            ),
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
            ("server_status.development_mode", "Development Mode"),
            ("server_status.proof_verifier_mode", "Proof Verifier Mode"),
            ("server_status.admin_auth_mode", "Admin Auth Mode"),
            (
                "server_status.dev_banner",
                "DEVELOPMENT MODE — proof verification disabled, do not use in production",
            ),
            (
                "server_status.dev_banner_dashboard",
                "This server is running with SOLAND_DEVELOPMENT_MODE=true. Proof verification is relaxed and admin endpoints accept any authenticated session.",
            ),
            // T6.2 §1 — conformance posture chips.
            ("server_status.conformance_label", "Conformance Posture"),
            ("server_status.verified_profiles", "Verified profiles"),
            ("server_status.claimed_profiles", "Self-claimed profiles"),
            (
                "server_status.experimental_features",
                "Experimental features",
            ),
            ("server_status.compat_surfaces", "Compatibility surfaces"),
            (
                "server_status.verified_blocked_dev",
                "Verified profiles are unavailable in dev mode — the relaxed proof verifier voids the verification claim.",
            ),
            (
                "server_status.experimental_warning",
                "Experimental: shape may change without notice. Do not rely on this in production.",
            ),
            // T6.2 §6 — dev-posture posture card on the server status page.
            (
                "server_status.dev_posture_title",
                "Weak runtime posture detected",
            ),
            (
                "server_status.dev_posture_verifier",
                "proof_verifier_mode = development (unsigned envelopes accepted)",
            ),
            (
                "server_status.dev_posture_admin",
                "admin_auth_mode = development (admin endpoints accept any authenticated session)",
            ),
            (
                "server_status.dev_posture_plaintext",
                "plaintext_visibility",
            ),
            // Round 4 — ServerDescribe v2 additional field labels.
            ("server_status.trust_domain", "Trust domain"),
            ("server_status.supported_operations", "Supported operations"),
            ("server_status.supported_bindings", "Supported bindings"),
            ("server_status.implemented_features", "Implemented features"),
            ("server_status.limits_label", "Limits"),
            ("server_status.rate_limit_label", "Rate limit"),
            (
                "server_status.dev_with_verified_title",
                "development_mode + verified_profiles is a contradiction",
            ),
            (
                "server_status.dev_with_verified_detail",
                "The relaxed dev-mode proof verifier voids every `verified_profiles` claim. The verification chain that backs those claims is unreachable while development_mode is on. Either clear verified_profiles or turn development_mode off.",
            ),
            // Round 4 — delivery-binding handover panel.
            (
                "delivery_binding.handover.title",
                "Delivery binding handover",
            ),
            (
                "delivery_binding.handover.subtitle",
                "When a recipient service hands over its binding to another service, the reducer emits one of the round-4 error codes below.",
            ),
            (
                "delivery_binding.handover.new_recipient",
                "new_recipient_service_did",
            ),
            ("delivery_binding.handover.frontier", "handover_frontier"),
            (
                "delivery_binding.handover.stale_explainer",
                "delivery_binding_stale (409): the recipient service rejected the envelope because its binding state has moved on. Retry against `new_recipient_service_did` at or after `handover_frontier`.",
            ),
            (
                "delivery_binding.handover.handed_over_explainer",
                "delivery_binding_handed_over (409): the recipient service has permanently handed delivery off. Submissions MUST be re-targeted to `new_recipient_service_did`.",
            ),
            (
                "delivery_binding.handover.historical_only_explainer",
                "historical_only (200 diagnostic): the response is a cached replay against a prior key state. It is informational only — NOT a fresh action.",
            ),
            // Round 4 — 3PID invite state machine.
            ("nav.invites_3pid", "3PID invites"),
            ("invites_3pid.title", "3PID invite state machine"),
            (
                "invites_3pid.subtitle",
                "Round 4 — third-party invites carry only oob_code commitments / lookup refs on the wire. Plaintext email/SMS NEVER appears here. Each row shows the current terminal state.",
            ),
            ("invites_3pid.invite_id", "Invite id"),
            ("invites_3pid.oob_mode", "OOB mode"),
            ("invites_3pid.verifier", "verification_service_did"),
            ("invites_3pid.terminal_state", "Terminal state"),
            ("invites_3pid.evidence", "Evidence"),
            ("invites_3pid.state_claimed", "claimed"),
            ("invites_3pid.state_send_failed", "send_failed"),
            (
                "invites_3pid.state_revoked_by_capability_loss",
                "revoked_by_capability_loss",
            ),
            (
                "invites_3pid.state_revoked_by_inviter_left",
                "revoked_by_inviter_left",
            ),
            (
                "invites_3pid.state_invalidated_by_rate_limit",
                "invalidated_by_rate_limit",
            ),
            (
                "invites_3pid.send_failed_warning",
                "send_failed is a TERMINAL state, not transient. The auth server never delivered the OOB code; the recipient cannot redeem this invite. Do not paper this over as a success.",
            ),
            (
                "invites_3pid.empty",
                "No 3PID invites in any terminal state.",
            ),
            // Round 4 — DID input validation.
            (
                "did_input.invalid",
                "Must match `^did:[a-z0-9]+:[^\\s]+$` (round-4 tightened method-name grammar — no `.`/`-`/`_`/`:` in the method segment).",
            ),
            // Round 4 — account deactivation federation status.
            (
                "actors.deactivation_federation_incomplete",
                "Federation fanout incomplete",
            ),
            (
                "actors.deactivation_federation_incomplete_detail",
                "The 7-domain local fanout has completed but at least one federated peer has NOT confirmed deactivation. Do NOT treat this principal as fully deactivated until cross-PS receipts arrive.",
            ),
            // T6.2 §2 — handle management.
            ("nav.handles", "Handles"),
            ("handles.title", "Handles"),
            (
                "handles.subtitle",
                "User handles projected from cx.handle.* cells. Inspect issuer, expiry, last reassignment, and the per-handle audit trail.",
            ),
            ("handles.search", "Search by handle, alias, or DID"),
            ("handles.canonical_uri", "Canonical URI"),
            ("handles.aliases", "Aliases"),
            ("handles.issuer", "Issuer DID"),
            ("handles.subject", "Subject DID"),
            ("handles.assigned_at", "Assigned at"),
            ("handles.expires_at", "Expires at"),
            ("handles.last_reassignment", "Last reassignment"),
            ("handles.status", "Status"),
            ("handles.empty", "No handles found"),
            ("handles.revoke", "Revoke"),
            ("handles.reassign", "Reassign"),
            ("handles.revoke_title", "Revoke handle?"),
            (
                "handles.revoke_body",
                "Publishes a cx.handle.revoke Move. The canonical URI will no longer resolve until reassigned. Audit-logged. Continue?",
            ),
            ("handles.revoke_ok", "Handle revoked"),
            ("handles.revoke_fail", "Failed to revoke handle"),
            ("handles.reassign_title", "Reassign handle"),
            (
                "handles.reassign_body",
                "Force-rebinds this canonical URI to a new subject DID. The previous owner loses ownership immediately. Audit-logged.",
            ),
            ("handles.new_subject_id", "New subject ID"),
            ("handles.reassign_ok", "Handle reassigned"),
            ("handles.reassign_fail", "Failed to reassign handle"),
            ("handles.detail_title", "Handle detail"),
            ("handles.audit_title", "Audit trail"),
            (
                "handles.audit_subtitle",
                "Every assignment, revoke, and reassignment recorded for this handle.",
            ),
            ("handles.audit_empty", "No audit events for this handle"),
            ("handles.audit_when", "When"),
            ("handles.audit_action", "Action"),
            ("handles.audit_actor", "Actor"),
            ("handles.audit_reason", "Reason"),
            // T6.2 §3 — delivery binding policy editor (realm-rework:
            // security boundary is now called "Realm").
            ("nav.delivery_binding", "Delivery binding"),
            ("delivery_binding.title", "Realm delivery binding policy"),
            ("delivery_binding.realm", "Realm"),
            // Legacy key retained so any stale call site keeps rendering.
            ("delivery_binding.space", "Realm"),
            ("delivery_binding.policy_title", "Effective policy"),
            (
                "delivery_binding.policy_subtitle",
                "Edit the recipient-service allow list and binding-source rules. policy_frontier is reducer-owned and read-only.",
            ),
            ("delivery_binding.policy_frontier", "Policy frontier"),
            ("delivery_binding.updated_at", "Updated at"),
            (
                "delivery_binding.binding_source_policy",
                "Binding source policy",
            ),
            (
                "delivery_binding.binding_source_hint",
                "Comma-separated list of accepted binding sources, e.g. explicit, invite, organization_policy.",
            ),
            (
                "delivery_binding.allowed_recipient_services",
                "Allowed recipient services",
            ),
            (
                "delivery_binding.allowed_empty",
                "No services in the allow list yet.",
            ),
            (
                "delivery_binding.allowed_edit_hint",
                "Comma-separated list of member_delivery_binding.recipient_service_did values. Add/remove inline.",
            ),
            ("delivery_binding.add_recipient", "Add"),
            ("delivery_binding.save", "Save policy"),
            (
                "delivery_binding.save_ok",
                "Delivery binding policy updated",
            ),
            ("delivery_binding.save_fail", "Failed to update policy"),
            ("delivery_binding.members_title", "Member routability"),
            (
                "delivery_binding.members_subtitle",
                "Each member's effective member_delivery_binding.recipient_service_did checked against the allow list above.",
            ),
            (
                "delivery_binding.members_empty",
                "No members in this realm.",
            ),
            ("delivery_binding.member_actor", "Actor"),
            ("delivery_binding.member_recipient", "Recipient service DID"),
            ("delivery_binding.member_status", "Delivery status"),
            ("delivery_binding.member_routability", "Routability"),
            ("delivery_binding.routable", "Routable"),
            ("delivery_binding.unroutable", "Not routable"),
            ("delivery_binding.link_graph_title", "Realm link graph"),
            (
                "delivery_binding.link_graph_subtitle",
                "Cross-realm trust edges and federated container Spaces.",
            ),
            (
                "delivery_binding.link_graph_placeholder",
                "Realm link graph coming soon.",
            ),
            // R5.2 — Realm link-graph admin page.
            ("nav.realm_links", "Realm links"),
            ("realm_links.title", "Realm link graph"),
            ("realm_links.realm", "Realm"),
            ("realm_links.outbound_title", "Outbound links"),
            (
                "realm_links.outbound_subtitle",
                "Typed cx.realm.link edges from this Realm to others (governed_by / discoverable_from / mirror_of …).",
            ),
            (
                "realm_links.outbound_empty",
                "No outbound links from this Realm.",
            ),
            ("realm_links.inbound_title", "Inbound links"),
            (
                "realm_links.inbound_subtitle",
                "Typed cx.realm.link edges from other Realms pointing at this one.",
            ),
            (
                "realm_links.inbound_empty",
                "No inbound links to this Realm.",
            ),
            ("realm_links.graph_title", "Graph visualisation"),
            (
                "realm_links.graph_subtitle",
                "Spatial layout of the link rows above.",
            ),
            (
                "realm_links.graph_placeholder",
                "Interactive graph visualisation coming soon — for now use the lists above.",
            ),
            ("realm_links.kind.governed_by", "governed by"),
            ("realm_links.kind.discoverable_from", "discoverable from"),
            ("realm_links.kind.mirror_of", "mirror of"),
            // T6.2 §4 — push route / device route inspector.
            ("nav.push_routes", "Push routes"),
            ("push_routes.title", "Push routes"),
            (
                "push_routes.subtitle",
                "cx.device.push_route cells grouped by principal. Push target ids are sensitive and stay collapsed until you explicitly reveal them.",
            ),
            ("push_routes.search", "Filter by principal id"),
            ("push_routes.principal", "Principal"),
            ("push_routes.device", "Device"),
            ("push_routes.transport", "Transport"),
            ("push_routes.cell_subject", "Cell subject"),
            ("push_routes.status", "Status"),
            ("push_routes.last_rotation", "Last rotation"),
            ("push_routes.target", "Push target id"),
            ("push_routes.empty", "No push routes for this principal."),
            ("push_routes.reveal", "Reveal"),
            ("push_routes.hide", "Hide"),
            // T6.2 §5 — capability constraint editor extras.
            ("capabilities.edit_title", "Edit capability"),
            ("capabilities.fields_write_allow", "Field write allow list"),
            ("capabilities.facets_allow", "Facet allow list"),
            ("capabilities.approval_required", "Approval required"),
            ("capabilities.csv_hint", "Comma-separated values"),
            ("capabilities.edit_ok", "Capability updated"),
            ("capabilities.edit_fail", "Failed to update capability"),
            ("nav.coauth_capabilities", "Authz Capabilities"),
            ("nav.spaces_admin", "Spaces (admin)"),
            ("nav.moderation_reports", "Moderation reports"),
            ("common.refresh", "Refresh"),
            ("common.previous", "Previous"),
            ("common.next", "Next"),
            ("common.confirm", "Confirm"),
            ("authz_caps.title", "Authz capability grants"),
            (
                "authz_caps.subtitle",
                "soland authz capability grants visible to the current admin scope. Filter by holder / peer / scope; revoke to publish a cx.cell.authz.capability.revoke Move.",
            ),
            ("authz_caps.holder", "Holder DID"),
            ("authz_caps.peer", "Peer DID"),
            ("authz_caps.scope", "Scope"),
            ("authz_caps.status", "Status"),
            ("authz_caps.granted_at", "Granted at"),
            ("authz_caps.expires_at", "Expires at"),
            ("authz_caps.revoke", "Revoke"),
            ("authz_caps.filter_holder", "Filter by holder DID"),
            ("authz_caps.filter_peer", "Filter by peer DID"),
            ("authz_caps.filter_scope", "Filter by scope"),
            ("authz_caps.empty_title", "No capability grants"),
            (
                "authz_caps.empty_subtitle",
                "soland reported no capability grants matching the current filter.",
            ),
            (
                "authz_caps.revoke_confirm_title",
                "Revoke capability grant?",
            ),
            (
                "authz_caps.revoke_confirm_body",
                "Publishes a revoke Move on the authz cell. Existing sessions resting on this capability will be re-checked on next anchor view. Audit-logged. Continue?",
            ),
            ("coauth_devices.title", "Account devices"),
            (
                "coauth_devices.subtitle",
                "Devices registered to this account. Revoke cascades to linked session grants on the soland side.",
            ),
            ("coauth_devices.device_id", "Device ID"),
            ("coauth_devices.display_name", "Display name"),
            ("coauth_devices.status", "Status"),
            ("coauth_devices.last_seen", "Last seen"),
            ("coauth_devices.linked_sessions", "Linked sessions"),
            ("coauth_devices.revoke", "Revoke"),
            ("coauth_devices.empty_title", "No devices registered"),
            (
                "coauth_devices.empty_subtitle",
                "coauth has not registered any device for this account.",
            ),
            ("coauth_devices.revoke_confirm_title", "Revoke device?"),
            (
                "coauth_devices.revoke_confirm_body",
                "Marks the device as revoked in coauth and cascade-revokes any session grants tied to it on the soland side. Audit-logged. Continue?",
            ),
            ("spaces_admin.title", "Spaces (admin)"),
            (
                "spaces_admin.subtitle",
                "Spaces visible to the current admin scope, backed by soland's admin describe surface.",
            ),
            ("spaces_admin.search_placeholder", "Search by id or name"),
            ("spaces_admin.id", "ID"),
            ("spaces_admin.name", "Name"),
            ("spaces_admin.members", "Members"),
            ("spaces_admin.health", "Health"),
            ("spaces_admin.created_at", "Created at"),
            ("spaces_admin.open_detail", "Open detail"),
            ("spaces_admin.open_hierarchy", "Hierarchy"),
            ("spaces_admin.empty_title", "No Spaces"),
            (
                "spaces_admin.empty_subtitle",
                "soland reported no Spaces matching the current filter for this admin scope.",
            ),
            ("space_hierarchy.title", "Hierarchy"),
            (
                "space_hierarchy.subtitle",
                "Immediate parent and direct children for this Space.",
            ),
            ("space_hierarchy.parent_label", "Parent"),
            ("space_hierarchy.this_label", "This"),
            ("space_hierarchy.child_label", "Child"),
            ("space_hierarchy.children_label", "Children"),
            ("space_hierarchy.empty_title", "No relations"),
            (
                "space_hierarchy.empty_subtitle",
                "This Space has no parent and no immediate children.",
            ),
            ("moderation_reports.title", "Moderation reports"),
            (
                "moderation_reports.subtitle",
                "Open moderation reports awaiting an admin decision. Resolve marks the matter closed; Dismiss records a no-op review.",
            ),
            ("moderation_reports.id", "Report ID"),
            ("moderation_reports.reporter", "Reporter"),
            ("moderation_reports.target", "Target"),
            ("moderation_reports.space", "Space"),
            ("moderation_reports.reason", "Reason"),
            ("moderation_reports.status", "Status"),
            ("moderation_reports.created_at", "Created at"),
            ("moderation_reports.resolve", "Resolve"),
            ("moderation_reports.dismiss", "Dismiss"),
            ("moderation_reports.filter_open", "Open"),
            ("moderation_reports.filter_resolved", "Resolved"),
            ("moderation_reports.filter_dismissed", "Dismissed"),
            ("moderation_reports.filter_all", "All"),
            ("moderation_reports.empty_title", "No moderation reports"),
            (
                "moderation_reports.empty_subtitle",
                "soland reported no moderation reports for the current filter.",
            ),
            (
                "moderation_reports.resolve_confirm_title",
                "Resolve report?",
            ),
            (
                "moderation_reports.resolve_confirm_body",
                "Records an admin Resolve decision on this report. Audit-logged. Continue?",
            ),
            ("moderation_reports.resolve_confirm_btn", "Confirm resolve"),
            (
                "moderation_reports.dismiss_confirm_title",
                "Dismiss report?",
            ),
            (
                "moderation_reports.dismiss_confirm_body",
                "Records an admin Dismiss decision on this report (no further action). Audit-logged. Continue?",
            ),
            ("moderation_reports.dismiss_confirm_btn", "Confirm dismiss"),
            ("nav.applets_admin", "Applets (admin)"),
            ("nav.agents_admin", "Agents (admin)"),
            ("nav.directory_admin", "Directory (admin)"),
            ("nav.federation_status", "Federation status"),
            ("nav.policy_editor", "Policy editor"),
            ("nav.starid_resolver", "Starid resolver"),
            ("starid_resolver.title", "Starid resolver"),
            (
                "starid_resolver.subtitle",
                "Read-only view of the upstream did:webvh writer — head version, witness count, and freshness.",
            ),
            ("starid_resolver.card_title", "Resolver status"),
            (
                "starid_resolver.card_subtitle",
                "Aggregate state reported by /api/v1/identity/describe.",
            ),
            ("starid_resolver.service_did", "Service DID"),
            ("starid_resolver.registry_mode", "Registry mode"),
            ("starid_resolver.protocol_version", "Protocol version"),
            ("starid_resolver.head_version_id", "Head version_id"),
            ("starid_resolver.witness_count", "Witness count"),
            ("starid_resolver.freshness", "Last activity"),
            ("starid_resolver.methods_label", "Supported methods"),
            ("starid_resolver.profiles_label", "Profiles"),
            ("starid_resolver.status_healthy", "Healthy"),
            ("starid_resolver.status_pending", "Pending witnesses"),
            ("starid_resolver.status_idle", "Idle"),
            ("starid_resolver.not_configured_badge", "Not configured"),
            (
                "starid_resolver.not_configured_hint",
                "Set starid_public_url in the deployment config to enable this panel.",
            ),
            (
                "starid_resolver.not_configured_body",
                "Starid is not configured for this deployment. The sodmin admin SPA reads the upstream resolver URL from the deployment config — once it is wired the panel will start mirroring the writer's status.",
            ),
            (
                "starid_resolver.docs_link",
                "Open Starid resolver docs \u{2192}",
            ),
            ("federation_status.title", "Federation status"),
            (
                "federation_status.subtitle",
                "Per-Space federation peers, last anchor pulled, and outbound replication queue depth.",
            ),
            ("federation_status.generated_at", "Generated at"),
            ("federation_status.space_id", "Space"),
            ("federation_status.peer", "Peer"),
            ("federation_status.health", "Health"),
            (
                "federation_status.last_anchor_pulled_at",
                "Last anchor pulled",
            ),
            ("federation_status.last_pushed_at", "Last pushed"),
            ("federation_status.outbound_queue_depth", "Outbound queue"),
            ("federation_status.empty_title", "No federation peers"),
            (
                "federation_status.empty_subtitle",
                "soland reported no federation peers for this scope, or the federation status endpoint is not yet wired.",
            ),
            ("policy_editor.title", "Space policy editor"),
            (
                "policy_editor.subtitle",
                "Edit cx.component.space.policy.v1 components. Submit constructs a cas-register Move.",
            ),
            ("policy_editor.history_visibility", "History visibility"),
            ("policy_editor.join_rule", "Join rule"),
            ("policy_editor.guest_access", "Guest access"),
            ("policy_editor.federate", "Federate"),
            ("policy_editor.encryption_algorithm", "Encryption algorithm"),
            ("policy_editor.note", "Commit note"),
            ("policy_editor.submit", "Submit policy update"),
            ("policy_editor.confirm_title", "Submit policy update?"),
            (
                "policy_editor.confirm_body",
                "Constructs and signs a cas-register Move on cx.component.space.policy.v1. Audit-logged. Continue?",
            ),
            ("policy_editor.confirm_btn", "Confirm submit"),
            ("applets_admin.title", "Applets (admin)"),
            (
                "applets_admin.subtitle",
                "soland-side applet registrations awaiting approval / revocation.",
            ),
            ("applets_admin.id", "ID"),
            ("applets_admin.name", "Name"),
            ("applets_admin.owner_did", "Owner DID"),
            ("applets_admin.status", "Status"),
            ("applets_admin.registered_at", "Registered at"),
            ("applets_admin.empty_title", "No applet registrations"),
            (
                "applets_admin.empty_subtitle",
                "soland reported no applet registrations, or the surface is not yet wired.",
            ),
            ("applets_admin.approve_confirm_title", "Approve applet?"),
            (
                "applets_admin.approve_confirm_body",
                "Marks the applet registration as Approved. Audit-logged.",
            ),
            ("applets_admin.revoke_confirm_title", "Revoke applet?"),
            (
                "applets_admin.revoke_confirm_body",
                "Marks the applet registration as Revoked. Audit-logged.",
            ),
            ("applets_admin.suspend_confirm_title", "Suspend applet?"),
            (
                "applets_admin.suspend_confirm_body",
                "Temporarily disables the applet without revoking its identity. Re-approve to resume. Audit-logged.",
            ),
            ("agents_admin.title", "Agents (admin)"),
            (
                "agents_admin.subtitle",
                "soland-side agent registrations awaiting approval / revocation.",
            ),
            ("agents_admin.id", "ID"),
            ("agents_admin.name", "Name"),
            ("agents_admin.owner_did", "Owner DID"),
            ("agents_admin.status", "Status"),
            ("agents_admin.registered_at", "Registered at"),
            ("agents_admin.empty_title", "No agent registrations"),
            (
                "agents_admin.empty_subtitle",
                "soland reported no agent registrations, or the surface is not yet wired.",
            ),
            ("agents_admin.approve_confirm_title", "Approve agent?"),
            (
                "agents_admin.approve_confirm_body",
                "Marks the agent registration as Approved. Audit-logged.",
            ),
            ("agents_admin.revoke_confirm_title", "Revoke agent?"),
            (
                "agents_admin.revoke_confirm_body",
                "Marks the agent registration as Revoked. Audit-logged.",
            ),
            ("agents_admin.suspend_confirm_title", "Suspend agent?"),
            (
                "agents_admin.suspend_confirm_body",
                "Temporarily disables the agent without revoking its identity. Re-approve to resume. Audit-logged.",
            ),
            ("directory_admin.title", "Directory (admin)"),
            (
                "directory_admin.subtitle",
                "soland-side directory entries awaiting approval / revocation.",
            ),
            ("directory_admin.entry_id", "Entry ID"),
            ("directory_admin.kind", "Kind"),
            ("directory_admin.label", "Label"),
            ("directory_admin.owner_did", "Owner DID"),
            ("directory_admin.status", "Status"),
            ("directory_admin.published_at", "Published at"),
            ("directory_admin.empty_title", "No directory entries"),
            (
                "directory_admin.empty_subtitle",
                "soland reported no directory entries, or the surface is not yet wired.",
            ),
            ("directory_admin.approve_confirm_title", "Approve entry?"),
            (
                "directory_admin.approve_confirm_body",
                "Marks the directory entry as Approved (publicly listed). Audit-logged.",
            ),
            ("directory_admin.revoke_confirm_title", "Revoke entry?"),
            (
                "directory_admin.revoke_confirm_body",
                "Marks the directory entry as Revoked (delisted). Audit-logged.",
            ),
            ("directory_admin.reject_confirm_title", "Reject entry?"),
            (
                "directory_admin.reject_confirm_body",
                "Rejects the directory entry — it will not be listed publicly. Audit-logged.",
            ),
            // ── CXP-0007 Circle admin (P3A.3 / P3A.6 / P3A.8) ─────────
            ("nav.circles", "Circles"),
            ("nav.section_circles", "Circles"),
            ("circle.list_title", "Circles"),
            (
                "circle.list_description",
                "Encrypted sub-boundaries scoped inside a Realm. Each Circle carries its own MLS group and a member list that is a strict subset of the parent Realm's membership.",
            ),
            ("circle.create", "Create Circle"),
            (
                "circle.create_description",
                "Create a new Circle inside an existing Realm. The parent Realm is immutable once set.",
            ),
            ("circle.id", "Circle ID"),
            ("circle.title", "Title"),
            ("circle.title_placeholder", "Trust & Safety"),
            ("circle.summary", "Summary"),
            ("circle.realm_id", "Realm"),
            (
                "circle.realm_id_hint",
                "ck:realm:... identifier of the parent security boundary. Required.",
            ),
            ("circle.filter_realm", "Filter by realm"),
            (
                "circle.filter_realm_hint",
                "Circles are scoped to a single Realm. Paste a ck:realm:... id to load its Circles.",
            ),
            ("circle.filter_apply", "Apply"),
            ("circle.directory_visibility", "Directory visibility"),
            ("circle.join_rule", "Join rule"),
            ("circle.history_visibility", "History visibility"),
            ("circle.encryption_profile", "Encryption profile"),
            ("circle.mls_group_ref", "MLS group ref"),
            ("circle.member_count", "Members"),
            ("circle.members_total", "Total members:"),
            ("circle.created_at", "Created"),
            ("circle.created_by", "Created by"),
            ("circle.updated_at", "Updated"),
            ("circle.state", "State"),
            ("circle.state_active", "Active"),
            ("circle.state_archived", "Archived"),
            ("circle.state_tombstoned", "Tombstoned"),
            ("circle.state_unknown", "Unknown"),
            ("circle.overview", "Overview"),
            ("circle.actions", "Actions"),
            ("circle.manage_members", "Manage members"),
            ("circle.manage_scope", "Manage scope"),
            (
                "circle.scope_description",
                "Rotate the bound MLS group for this Circle. Receipts fan out into the audit log even when no membership changes accompany the rotation.",
            ),
            (
                "circle.scope_hint",
                "Full MLS-key rotation lands when soland P2A.4 is complete. Today this emits a cx.circle.update Move so authz hooks fire.",
            ),
            ("circle.rotate_scope", "Rotate MLS scope"),
            ("circle.archive", "Archive"),
            ("circle.tombstone", "Tombstone"),
            ("circle.add_member", "Add member"),
            ("circle.remove_member", "Remove"),
            ("circle.actor_id", "Actor ID"),
            ("circle.member_state", "State"),
            ("circle.empty_members", "No members yet."),
            ("circle.empty_title", "No Circles in this Realm"),
            (
                "circle.empty_description",
                "Use Create Circle to add the first encrypted sub-boundary.",
            ),
            ("circle.empty_no_realm_title", "Pick a Realm to inspect"),
            (
                "circle.empty_no_realm_description",
                "Circles are always scoped to a single Realm. Enter a ck:realm:... id above to load its Circles.",
            ),
            (
                "circle.subset_hint",
                "Members must already belong to the parent Realm. soland rejects out-of-subset adds with `circle_member_must_be_realm_member`.",
            ),
            ("circle.error_required", "realm_id and title are required."),
            ("circle.error_actor_required", "actor_id is required."),
            ("circle.created_toast", "Circle created."),
            ("circle.archived_toast", "Circle archived."),
            (
                "circle.tombstoned_toast",
                "Circle tombstoned. Terminal state.",
            ),
            (
                "circle.scope_rotated_toast",
                "Circle scope rotation acknowledged.",
            ),
            ("circle.member_added_toast", "Member added to Circle."),
            ("circle.member_removed_toast", "Member removed from Circle."),
            ("circle.confirm_archive_title", "Archive this Circle?"),
            (
                "circle.confirm_archive_body",
                "Archived Circles reject all member changes and ordinary writes. Scope rotation is still allowed. Reversible while inside the soft-delete window.",
            ),
            ("circle.confirm_tombstone_title", "Tombstone this Circle?"),
            (
                "circle.confirm_tombstone_body",
                "Tombstoned Circles are terminal: no further admin actions are accepted and member state is forever frozen. Erasure receipts fan out per CXP-0007.",
            ),
            // D.1 — plural `circles.*` namespace used by the listing
            // chrome (title bar, create button, member limit hints).
            // Keep alongside the existing singular `circle.*` keys so
            // grep finds either spelling.
            ("circles.title", "Circles"),
            ("circles.create", "Create Circle"),
            (
                "circles.member_limit",
                "Members must already belong to the parent Realm.",
            ),
            // B.7 — Export CSV button label, shared across the
            // agents / devices / spaces list pages.
            ("common.export_csv", "Export CSV"),
            ("common.enabled", "Enabled"),
            ("common.disabled", "Disabled"),
            // P3A.6 — Principal Control vs Collaboration Realm
            ("realm.principal_control", "Principal control"),
            ("realm.collaboration", "Collaboration"),
            ("realm.classification", "Realm classification"),
            (
                "realm.classification_hint",
                "Principal-control Realms back DID issuance and recovery; Collaboration Realms scope content. The class is immutable after create per CXP-0007.",
            ),
            // P3A.5 — audit effective scope + Circle event filters
            ("audit.effective_scope", "Effective scope"),
            ("audit.scope_realm", "Realm"),
            ("audit.scope_circle", "Circle"),
            ("audit.scope_jump", "Jump to scope"),
            ("audit.filter_event_kind", "Event kind"),
            ("audit.filter_event_kind_circle_create", "cx.circle.create"),
            ("audit.filter_event_kind_circle_update", "cx.circle.update"),
            (
                "audit.filter_event_kind_circle_archive",
                "cx.circle.archive",
            ),
            (
                "audit.filter_event_kind_circle_tombstone",
                "cx.circle.tombstone",
            ),
            (
                "audit.filter_event_kind_circle_member_state",
                "cx.circle.member.state",
            ),
            (
                "audit.filter_event_kind_circle_capability_grant",
                "cx.circle.capability.grant",
            ),
            (
                "audit.filter_event_kind_circle_capability_revoke",
                "cx.circle.capability.revoke",
            ),
            // P3A.4 — capability editor (cx.circle.*) labels
            ("capability.cx_circle_create", "cx.circle.create"),
            ("capability.cx_circle_manage", "cx.circle.manage"),
            ("capability.cx_circle_member_add", "cx.circle.member.add"),
            (
                "capability.cx_circle_member_manage",
                "cx.circle.member.manage",
            ),
            (
                "capability.cx_circle_member_add_others",
                "cx.circle.member.add.others",
            ),
            ("capability.cx_circle_audit", "cx.circle.audit"),
            (
                "capability.cx_circle_section",
                "Circle capabilities (CXP-0007)",
            ),
            ("capability.allowed_circle_ids", "Allowed circle IDs"),
            (
                "capability.allowed_circle_ids_hint",
                "Comma-separated ck:circle:... ids. Required for cx.circle.manage, cx.circle.member.manage, and cx.circle.member.add.others.",
            ),
            // P3A.8 — CXP-0007 error codes (returned by reducer)
            (
                "error.circle_realm_mismatch",
                "Circle realm mismatch — the requested action targets a different Realm than the Circle is bound to.",
            ),
            (
                "error.circle_member_must_be_realm_member",
                "Member must already belong to the parent Realm.",
            ),
            (
                "error.circle_not_active",
                "Circle is not active — archived or tombstoned Circles reject this action.",
            ),
            (
                "error.circle_already_terminal",
                "Circle is already in a terminal state (archived or tombstoned).",
            ),
            (
                "error.circle_capability_denied",
                "Required cx.circle.* capability is missing or scoped to a different Circle ref.",
            ),
            (
                "error.circle_scope_rotation_in_progress",
                "Another scope rotation is in flight for this Circle.",
            ),
            // P5 — /agents/personal page strings.
            ("agents.personal.title", "Personal agents"),
            (
                "agents.personal.subtitle",
                "Controller-self native agents (CXP-0008)",
            ),
            ("agents.personal.provision", "Provision new agent"),
            ("agents.personal.deactivate", "Deactivate"),
            ("agents.personal.pause", "Pause"),
            ("agents.personal.resume", "Resume"),
            ("agents.personal.rotate_key", "Rotate key"),
            (
                "agents.personal.no_agents",
                "No personal agents provisioned yet.",
            ),
            ("agents.personal.controller_did", "Controller DID"),
            ("agents.personal.display_name", "Display name (optional)"),
            ("agents.personal.step_label", "Step {step}/3"),
            ("agents.personal.identity", "Identity"),
            ("agents.personal.agent_keys", "Agent keys"),
            ("agents.personal.capability_grants", "Capability grants"),
            ("agents.personal.required_capability", "Required capability"),
            // R3 — Agent FSM list (UI-1).
            ("agents.title", "Agents"),
            (
                "agents.subtitle",
                "Native + personal agents — active / paused / deactivated.",
            ),
            ("agents.id", "Agent ID"),
            ("agents.name", "Name"),
            ("agents.owner_id", "Owner"),
            ("agents.agent_type", "Type"),
            ("agents.status", "Status"),
            ("agents.last_active_at", "Last active"),
            ("agents.no_agents", "No agents."),
            ("agents.pause", "Pause"),
            ("agents.resume", "Resume"),
            ("agents.rotate_key", "Rotate key"),
            ("agents.grants", "Grants"),
            ("agents.deactivate", "Deactivate"),
            ("agents.deactivate_title", "Deactivate agent?"),
            (
                "agents.deactivate_body",
                "This revokes the agent's keys, capability grants, and runtime endpoints. Type DEACTIVATE to confirm.",
            ),
            // R3 — Agent draft / action_request lifecycle (UI-2 stub).
            ("agents.lifecycle.title", "Agent draft / action lifecycle"),
            (
                "agents.lifecycle.subtitle",
                "Drafts, action_request, action_approve, action_reject. Coming in R3.1.",
            ),
            (
                "agents.lifecycle.placeholder",
                "No draft/action data plumbed yet.",
            ),
            // R3 — Realm media_service editor (UI-3).
            ("media_service.title", "Realm media service"),
            (
                "media_service.subtitle",
                "Foci list — pick one or more media services (livekit, mediasoup, janus, cokret-native, moq-relay).",
            ),
            ("media_service.foci_label", "Foci"),
            ("media_service.empty", "No foci configured for this Realm."),
            ("media_service.add", "Add focus"),
            ("media_service.focus_id", "id"),
            ("media_service.focus_type", "type"),
            ("media_service.focus_connect_url", "connect_url"),
            ("media_service.focus_service_did", "service_did"),
            ("media_service.focus_regions", "regions (comma-separated)"),
            ("media_service.remove", "Remove"),
            ("media_service.save", "Save media_service"),
            ("media_service.save_ok", "media_service saved"),
            ("media_service.save_fail", "Failed to save media_service"),
            (
                "media_service.legacy_banner_title",
                "Legacy single-endpoint media_service detected",
            ),
            (
                "media_service.legacy_banner_body",
                "The sfu_endpoint field is being replaced by foci[]. Pre-fill the first focus from the legacy endpoint, fan out additional services, then save to migrate.",
            ),
            // R3 — Recovery policy view (UI-4 stub).
            ("recovery.title", "Recovery policies"),
            (
                "recovery.subtitle",
                "List, inspect, and rotate cx.coauth.recovery.* policies. Receipts include proof_summary[].",
            ),
            ("recovery.list_title", "Policies"),
            ("recovery.policy_id", "Policy id"),
            ("recovery.policy_type", "Type"),
            ("recovery.policy_status", "Status"),
            ("recovery.policy_actions", "Actions"),
            ("recovery.inspect", "Inspect"),
            ("recovery.rotate", "Rotate"),
            ("recovery.receipts_title", "Recovery receipts"),
            (
                "recovery.receipts_subtitle",
                "Each receipt's proof_summary[] lists the proofs the coauth recovery flow accepted.",
            ),
            (
                "recovery.placeholder",
                "Recovery data API ships in R3.1 — surface stubbed.",
            ),
            // R3 — Handle homograph inline warnings (UI-5).
            (
                "error.handle_homograph_forbidden",
                "This handle would be rejected as a homograph (handle_homograph_forbidden).",
            ),
            (
                "error.handle_homograph_out_of_range",
                "Handle localpart must be 1..=128 bytes.",
            ),
            (
                "error.handle_homograph_zero_width",
                "Handle contains zero-width or bidi-control codepoints — rejected.",
            ),
            (
                "error.handle_homograph_script_mixed",
                "Handle mixes Latin and non-Latin scripts — rejected as a homograph.",
            ),
            (
                "error.handle_homograph_confusable",
                "Handle contains codepoints that fold to ASCII look-alikes — rejected.",
            ),
            // R3 — Server profile toggles (UI-6).
            (
                "server_status.profile.media_service_binding",
                "media_service_binding.v1 — Realm-level foci binding profile.",
            ),
            (
                "server_status.profile.accountable_principals_strict_reject",
                "accountable_principals.strict_reject.v1 — fail-closed on missing accountability grants.",
            ),
            (
                "server_status.profile.key_backup_memory_hard",
                "key_backup.memory_hard.v1 — Argon2id memory-hard key backup KDF.",
            ),
            // R3 — New wire error codes (UI-7).
            (
                "error.pairing_request_expired",
                "The pairing request expired before approval. Restart the pairing flow.",
            ),
            (
                "error.proof_invalid",
                "Submitted proof failed verification.",
            ),
            (
                "error.agent_paused",
                "The agent is paused. Resume it before retrying.",
            ),
            (
                "error.agent_deactivated",
                "The agent is deactivated and cannot accept new work.",
            ),
            (
                "error.accountability_grant_missing",
                "Action requires an active accountability_grant from the controller.",
            ),
            (
                "error.participant_binding_invalid",
                "Participant binding does not match the Realm's allowed recipient services.",
            ),
            (
                "error.legacy_single_endpoint_media_service",
                "Realm still uses the legacy sfu_endpoint field — migrate to foci[].",
            ),
            // P5 — /key-backup page strings.
            ("key_backup.title", "Key backup"),
            (
                "key_backup.subtitle",
                "Server-side cross-signing key backup",
            ),
            ("key_backup.no_backup", "No key backup found on server."),
            ("key_backup.backup_version", "Backup version"),
            ("key_backup.algorithm", "Algorithm"),
            ("key_backup.created", "Created"),
            ("key_backup.delete", "Delete backup"),
            (
                "key_backup.delete_confirm",
                "Delete this key backup version?",
            ),
            ("key_backup.refresh", "Refresh"),
            // R3.1 (HDLREN-1) — handle normalize error tags.
            ("error.handle_empty", "Handle is empty after trimming."),
            (
                "error.handle_malformed",
                "Handle could not be parsed as <localpart>:<domain> — canonical wire form is required.",
            ),
            // R3.1 (MID-1..3, ROST-1..2) — Realm members + identity diagnostic.
            ("spaces.identity", "Identity"),
            ("spaces.membership", "Membership"),
            (
                "spaces.members_subtitle",
                "Handles and display names are sourced from the effective MemberIdentity, not the legacy sync members[] row.",
            ),
            ("spaces.identity_pending", "Identity pending decryption"),
            ("spaces.members_showing", "Showing"),
            ("spaces.members_of_many", "of many"),
            ("spaces.members_load_more", "Load more"),
            ("spaces.open_identity_audit", "Open identity audit"),
            ("realm_identity_audit.title", "Identity audit"),
            (
                "realm_identity_audit.subtitle",
                "Per-actor effective cx.member.identity.update event ids and member_display_state_digest projection.",
            ),
            ("realm_identity_audit.stub_title", "Data plumbing pending"),
            (
                "realm_identity_audit.stub_body",
                "This diagnostic surface lights up once yougen's MLS decrypt pipeline (MID-4) lands. The page renders the empty scaffold today so the route is reachable from the Realm detail card.",
            ),
            ("realm_identity_audit.actors_title", "Actors"),
            (
                "realm_identity_audit.actors_subtitle",
                "One row per actor in this Realm. cache_drift is set when the local projection disagrees with the roster row.",
            ),
            ("realm_identity_audit.col_actor", "Actor DID"),
            ("realm_identity_audit.col_primary_handle", "Primary handle"),
            ("realm_identity_audit.col_display_name", "Display name"),
            ("realm_identity_audit.col_event_ids", "Effective event ids"),
            (
                "realm_identity_audit.col_state_digest",
                "member_display_state_digest",
            ),
            ("realm_identity_audit.col_cache", "Cache"),
            ("realm_identity_audit.cache_drift", "drift"),
            ("realm_identity_audit.cache_ok", "ok"),
            ("realm_identity_audit.empty", "No actors observed yet."),
            // R3.2 (UI-SOD-4) — Subject → Handles directory page.
            ("nav.handles_by_subject", "Subject → Handles"),
            ("handles_by_subject.title", "Subject → Handles"),
            (
                "handles_by_subject.subtitle",
                "Reverse-lookup the signed handle claims currently visible for a holder/principal DID (cx.directory.list_handles_for_subject).",
            ),
            ("handles_by_subject.lookup_title", "Look up a subject"),
            (
                "handles_by_subject.lookup_body",
                "Enter a holder/principal DID. MemberIdentity no longer carries a handle — handles come from signed cx.schema.handle_claim.v1 evidence.",
            ),
            ("handles_by_subject.subject_id", "Subject ID"),
            ("handles_by_subject.lookup", "Look up"),
            (
                "handles_by_subject.enter_did",
                "Enter a subject DID above to list its visible handle claims.",
            ),
            ("handles_by_subject.results_title", "Visible handle claims"),
            (
                "handles_by_subject.primary_handle",
                "Primary handle (§3.2.1)",
            ),
            ("handles_by_subject.primary_badge", "primary"),
            ("handles_by_subject.col_handle", "Handle"),
            ("handles_by_subject.col_issuer", "Issuer DID"),
            ("handles_by_subject.col_binding", "Binding"),
            ("handles_by_subject.col_expires", "Expires at"),
            ("handles_by_subject.col_why", "Why?"),
            ("handles_by_subject.why", "Why am I seeing this?"),
            ("handles_by_subject.why_issuer", "issuer"),
            ("handles_by_subject.why_binding", "binding_state"),
            ("handles_by_subject.why_created", "created_at"),
            (
                "handles_by_subject.no_claims",
                "No visible handle claims for this subject in the current context.",
            ),
            (
                "handles_by_subject.has_more",
                "More claims are available — refine the context to narrow the result set.",
            ),
            ("handles_by_subject.binding_verified", "verified"),
            ("handles_by_subject.binding_pending", "pending"),
            ("handles_by_subject.binding_revoked", "revoked"),
            ("handles_by_subject.binding_expired", "expired"),
            // R3.2 (UI-SOD-5) — handle changed since hint.
            ("handle_change.changed_since", "handle changed since"),
            ("handle_change.now", "now"),
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
            ("nav.consent", "授权同意"),
            ("nav.covered_frontier", "Covered Frontier"),
            ("nav.components", "组件注册表"),
            ("nav.signing_keys", "签名密钥"),
            ("nav.multisig", "多签"),
            ("signing_keys.title", "Anchorer 签名密钥"),
            (
                "signing_keys.subtitle",
                "AnchorerWorker 签名密钥来源、验证方法及轮换控制。",
            ),
            ("signing_keys.origin", "来源"),
            ("signing_keys.origin.configured", "已配置"),
            ("signing_keys.origin.ephemeral", "临时"),
            ("signing_keys.verification_method", "验证方法"),
            ("signing_keys.did", "DID"),
            ("signing_keys.kid", "kid"),
            ("signing_keys.algorithm", "算法"),
            ("signing_keys.last_rotated", "上次轮换"),
            ("signing_keys.rotate", "轮换签名密钥"),
            (
                "signing_keys.rotate_confirm_title",
                "轮换 AnchorerWorker 签名密钥？",
            ),
            (
                "signing_keys.rotate_confirm_body",
                "将生成新密钥并原子地重新绑定 AnchorerWorker；进行中的 Anchor 会用新密钥重新签名。该操作记入审计。继续？",
            ),
            (
                "signing_keys.ephemeral_warning",
                "Anchorer 当前使用临时密钥签名。该密钥在 worker 重启后即丢失且无法原地轮换 —— 在升级到生产环境前，请使用已配置密钥（PEM / KMS）重新部署 principal-server。",
            ),
            (
                "signing_keys.read_only_note",
                "（只读 —— 必须先用已配置密钥重新部署才能轮换）",
            ),
            (
                "signing_keys.rotate_disabled_hint",
                "当前签名密钥来源不支持轮换。请先配置稳定的 PEM / KMS 签名后端并重新部署。",
            ),
            (
                "signing_keys.rotate_enabled_hint",
                "轮换将生成新密钥并原子地切换 worker 签名身份，返回新的验证方法 id。该操作记入审计。",
            ),
            ("multisig.title", "多签待办"),
            (
                "multisig.subtitle",
                "等待 anchorer cell 成员达到阈值部分签名的 Anchor。",
            ),
            ("multisig.col_anchor", "Anchor ID"),
            ("multisig.col_threshold", "阈值"),
            ("multisig.col_collected", "已收集"),
            ("multisig.col_missing", "缺失签名者"),
            ("multisig.col_state_root", "State root"),
            ("multisig.col_created", "创建时间"),
            ("multisig.submit_partial", "提交我的部分签名"),
            (
                "multisig.empty",
                "本 Space 的所有 Anchor 已达阈值并完成组装。",
            ),
            ("consent.title", "授权同意列表"),
            ("consent.subtitle", "consent cell or-set 值的只读 join 投影"),
            ("consent.filter_holder", "按持有者 DID 过滤"),
            ("consent.holder_did", "持有者 DID"),
            ("consent.peer_did", "对端 DID"),
            ("consent.scope", "Scope"),
            ("consent.status", "状态"),
            ("consent.created", "创建时间"),
            ("consent.no_grants", "当前过滤条件下没有授权记录。"),
            ("components.title", "组件注册表"),
            (
                "components.subtitle",
                "服务器范围 component_type / cell_family 注册表，含 criticality 和版本 drift 指示。",
            ),
            ("components.col_component", "组件"),
            ("components.col_family", "Cell 家族"),
            ("components.col_criticality", "关键级别"),
            ("components.col_spec", "Spec"),
            ("components.col_impl", "Impl"),
            ("components.col_status", "状态"),
            ("components.col_drift", "Drift"),
            ("components.empty", "服务器未报告任何组件。"),
            (
                "components.drift_alert",
                "{count} 个组件 spec / impl 版本不一致。",
            ),
            ("covered_frontier.title", "covered_frontier 滞后"),
            (
                "covered_frontier.subtitle",
                "Governance frontier 与 MLS group epoch 对比 —— Move 确认滞后。",
            ),
            ("covered_frontier.lag", "滞后"),
            ("covered_frontier.mls_epoch", "MLS epoch"),
            (
                "covered_frontier.governance_size",
                "Governance frontier 大小",
            ),
            ("covered_frontier.covered_size", "Covered frontier 大小"),
            ("covered_frontier.latest_anchor", "最新 anchor"),
            (
                "covered_frontier.last_covered",
                "上次 covered_frontier 更新",
            ),
            (
                "covered_frontier.warn",
                "滞后已超过阈值；请排查 MLS group 健康度（成员掉线 / KeyPackage 过期）。",
            ),
            ("auth.footer", "sodmin - Cokret 管理后台"),
            ("auth.completing_login", "正在完成登录..."),
            ("dashboard.welcome", "欢迎使用 Cokret Admin"),
            ("dashboard.active_users", "活跃 Actor"),
            (
                "dashboard.spec_description",
                "Cokret 协议 profile、功能和 conformance 覆盖",
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
            ("coauth.audit_log.filter_actor_placeholder", "actor user id"),
            ("coauth.audit_log.filter_target_type", "目标类型"),
            (
                "coauth.audit_log.filter_target_type_placeholder",
                "如 account、session",
            ),
            ("coauth.audit_log.filter_target_id", "目标 ID"),
            ("coauth.audit_log.filter_target_id_placeholder", "target id"),
            ("coauth.audit_log.filter_since", "起始时间"),
            ("coauth.audit_log.filter_until", "截止时间"),
            ("coauth.audit_log.filter_apply", "应用"),
            ("coauth.audit_log.filter_reset", "重置"),
            ("server_status.title", "服务状态"),
            ("server_status.subtitle", "各 cokret 后端的健康与能力快照"),
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
            ("server_status.development_mode", "开发模式"),
            ("server_status.proof_verifier_mode", "签名验证模式"),
            ("server_status.admin_auth_mode", "管理认证模式"),
            (
                "server_status.dev_banner",
                "开发模式 — 签名验证已关闭，请勿用于生产",
            ),
            (
                "server_status.dev_banner_dashboard",
                "当前服务以 SOLAND_DEVELOPMENT_MODE=true 启动：签名验证被放宽，管理端点接受任何已认证会话。",
            ),
            // T6.2 §1 — 协议一致性徽标。
            ("server_status.conformance_label", "协议一致性"),
            ("server_status.verified_profiles", "已验证 Profile"),
            ("server_status.claimed_profiles", "自声明 Profile"),
            ("server_status.experimental_features", "实验性特性"),
            ("server_status.compat_surfaces", "兼容性接口"),
            (
                "server_status.verified_blocked_dev",
                "开发模式下已验证 Profile 不可用——放宽的签名校验使验证失效。",
            ),
            (
                "server_status.experimental_warning",
                "实验性：接口可能随时变化，不建议在生产中使用。",
            ),
            // T6.2 §6 — 服务器状态页的开发模式风险卡。
            ("server_status.dev_posture_title", "检测到弱运行态势"),
            (
                "server_status.dev_posture_verifier",
                "proof_verifier_mode = development（接受未签名事件）",
            ),
            (
                "server_status.dev_posture_admin",
                "admin_auth_mode = development（管理端点接受任何已认证会话）",
            ),
            (
                "server_status.dev_posture_plaintext",
                "plaintext_visibility",
            ),
            // Round 4 — ServerDescribe v2 新增字段。
            ("server_status.trust_domain", "Trust domain"),
            ("server_status.supported_operations", "支持的 operation"),
            ("server_status.supported_bindings", "支持的 binding"),
            ("server_status.implemented_features", "已实现的特性"),
            ("server_status.limits_label", "限制 (limits)"),
            ("server_status.rate_limit_label", "限速 (rate_limit)"),
            (
                "server_status.dev_with_verified_title",
                "development_mode 与 verified_profiles 同时为真——矛盾配置",
            ),
            (
                "server_status.dev_with_verified_detail",
                "dev 模式下宽松的 proof verifier 会作废所有 verified_profiles 声明。先清空 verified_profiles，或关闭 development_mode。",
            ),
            (
                "delivery_binding.handover.title",
                "Delivery binding handover",
            ),
            (
                "delivery_binding.handover.subtitle",
                "接收方服务把 delivery binding 移交给其它服务时，reducer 会发出下列 Round 4 错误码。",
            ),
            (
                "delivery_binding.handover.new_recipient",
                "new_recipient_service_did",
            ),
            ("delivery_binding.handover.frontier", "handover_frontier"),
            (
                "delivery_binding.handover.stale_explainer",
                "delivery_binding_stale (409)：接收方因 binding 状态已推进而拒绝。重试请使用 new_recipient_service_did，且不早于 handover_frontier。",
            ),
            (
                "delivery_binding.handover.handed_over_explainer",
                "delivery_binding_handed_over (409)：接收方已永久移交 delivery，必须改投递到 new_recipient_service_did。",
            ),
            (
                "delivery_binding.handover.historical_only_explainer",
                "historical_only (200 diagnostic)：响应是针对旧 key state 的缓存回放，仅为信息——不代表新动作。",
            ),
            ("nav.invites_3pid", "3PID 邀请"),
            ("invites_3pid.title", "3PID invite state machine"),
            (
                "invites_3pid.subtitle",
                "Round 4 — 第三方邀请在 wire 上只携带 oob 承诺 / lookup ref，明文 email/SMS 永不出现。每行显示当前终态。",
            ),
            ("invites_3pid.invite_id", "Invite id"),
            ("invites_3pid.oob_mode", "OOB 模式"),
            ("invites_3pid.verifier", "verification_service_did"),
            ("invites_3pid.terminal_state", "终态"),
            ("invites_3pid.evidence", "证据"),
            ("invites_3pid.state_claimed", "claimed"),
            ("invites_3pid.state_send_failed", "send_failed"),
            (
                "invites_3pid.state_revoked_by_capability_loss",
                "revoked_by_capability_loss",
            ),
            (
                "invites_3pid.state_revoked_by_inviter_left",
                "revoked_by_inviter_left",
            ),
            (
                "invites_3pid.state_invalidated_by_rate_limit",
                "invalidated_by_rate_limit",
            ),
            (
                "invites_3pid.send_failed_warning",
                "send_failed 是终态，不是临时状态。auth server 从未投递 OOB code，对方无法兑换此邀请。严禁伪装成功。",
            ),
            ("invites_3pid.empty", "目前没有处于任何终态的 3PID invite。"),
            (
                "did_input.invalid",
                "必须匹配 `^did:[a-z0-9]+:[^\\s]+$`（Round 4 收紧后的 method-name，不允许 `.`/`-`/`_`/`:`）。",
            ),
            (
                "actors.deactivation_federation_incomplete",
                "联邦端注销未完成",
            ),
            (
                "actors.deactivation_federation_incomplete_detail",
                "本地 7 域 fanout 已完成，但仍有至少一个联邦对端未确认注销。在跨 PS 回执到齐前，不可将该 principal 视为已完全注销。",
            ),
            // T6.2 §2 — Handle 管理。
            ("nav.handles", "Handle"),
            ("handles.title", "Handle"),
            (
                "handles.subtitle",
                "从 cx.handle.* cell 投影出的用户 handle：签发者、过期时间、最近一次 reassignment 与逐条审计。",
            ),
            ("handles.search", "按 handle、别名或 DID 搜索"),
            ("handles.canonical_uri", "Canonical URI"),
            ("handles.aliases", "别名"),
            ("handles.issuer", "签发者 DID"),
            ("handles.subject", "主体 DID"),
            ("handles.assigned_at", "签发时间"),
            ("handles.expires_at", "过期时间"),
            ("handles.last_reassignment", "最近 Reassignment"),
            ("handles.status", "状态"),
            ("handles.empty", "暂无 handle"),
            ("handles.revoke", "撤销"),
            ("handles.reassign", "重新分配"),
            ("handles.revoke_title", "撤销 handle？"),
            (
                "handles.revoke_body",
                "将发布 cx.handle.revoke Move，该 canonical URI 在重新分配前不可解析。已记入审计。是否继续？",
            ),
            ("handles.revoke_ok", "已撤销 handle"),
            ("handles.revoke_fail", "撤销 handle 失败"),
            ("handles.reassign_title", "重新分配 handle"),
            (
                "handles.reassign_body",
                "强制将该 canonical URI 绑定到新的主体 DID，原所有者将立即失去所有权。已记入审计。",
            ),
            ("handles.new_subject_id", "新主体 ID"),
            ("handles.reassign_ok", "已重新分配 handle"),
            ("handles.reassign_fail", "重新分配 handle 失败"),
            ("handles.detail_title", "Handle 详情"),
            ("handles.audit_title", "审计轨迹"),
            (
                "handles.audit_subtitle",
                "记录此 handle 的所有签发、撤销与 reassignment。",
            ),
            ("handles.audit_empty", "暂无审计事件"),
            ("handles.audit_when", "时间"),
            ("handles.audit_action", "操作"),
            ("handles.audit_actor", "执行者"),
            ("handles.audit_reason", "原因"),
            // T6.2 §3 — Delivery binding policy 编辑器（realm-rework：安全边界改称 Realm）。
            ("nav.delivery_binding", "投递绑定"),
            ("delivery_binding.title", "Realm 投递绑定策略"),
            ("delivery_binding.realm", "Realm"),
            // 保留旧 key，避免遗留调用点报错。
            ("delivery_binding.space", "Realm"),
            ("delivery_binding.policy_title", "当前策略"),
            (
                "delivery_binding.policy_subtitle",
                "编辑 recipient-service 允许清单与 binding_source 规则。policy_frontier 由 reducer 写入，只读。",
            ),
            ("delivery_binding.policy_frontier", "Policy Frontier"),
            ("delivery_binding.updated_at", "更新时间"),
            (
                "delivery_binding.binding_source_policy",
                "Binding Source Policy",
            ),
            (
                "delivery_binding.binding_source_hint",
                "接受的 binding source 用逗号分隔，如 explicit, invite, organization_policy。",
            ),
            (
                "delivery_binding.allowed_recipient_services",
                "允许的 recipient 服务",
            ),
            ("delivery_binding.allowed_empty", "允许清单为空。"),
            (
                "delivery_binding.allowed_edit_hint",
                "用逗号分隔的 member_delivery_binding.recipient_service_did 列表，可直接增删。",
            ),
            ("delivery_binding.add_recipient", "添加"),
            ("delivery_binding.save", "保存策略"),
            ("delivery_binding.save_ok", "已更新投递绑定策略"),
            ("delivery_binding.save_fail", "更新策略失败"),
            ("delivery_binding.members_title", "成员可达性"),
            (
                "delivery_binding.members_subtitle",
                "对照上方允许清单，逐个检查成员的 member_delivery_binding.recipient_service_did。",
            ),
            ("delivery_binding.members_empty", "此 Realm 暂无成员。"),
            ("delivery_binding.member_actor", "成员"),
            ("delivery_binding.member_recipient", "Recipient 服务 DID"),
            ("delivery_binding.member_status", "投递状态"),
            ("delivery_binding.member_routability", "是否可达"),
            ("delivery_binding.routable", "可达"),
            ("delivery_binding.unroutable", "不可达"),
            ("delivery_binding.link_graph_title", "Realm 链接图"),
            (
                "delivery_binding.link_graph_subtitle",
                "跨 Realm 的信任边及联邦容器 Space。",
            ),
            (
                "delivery_binding.link_graph_placeholder",
                "Realm 链接图开发中。",
            ),
            // R5.2 — Realm 链接图管理页。
            ("nav.realm_links", "Realm 链接"),
            ("realm_links.title", "Realm 链接图"),
            ("realm_links.realm", "Realm"),
            ("realm_links.outbound_title", "出向链接"),
            (
                "realm_links.outbound_subtitle",
                "从本 Realm 指向其他 Realm 的 cx.realm.link 类型边（governed_by / discoverable_from / mirror_of …）。",
            ),
            ("realm_links.outbound_empty", "本 Realm 没有出向链接。"),
            ("realm_links.inbound_title", "入向链接"),
            (
                "realm_links.inbound_subtitle",
                "其他 Realm 指向本 Realm 的 cx.realm.link 类型边。",
            ),
            ("realm_links.inbound_empty", "本 Realm 没有入向链接。"),
            ("realm_links.graph_title", "图形可视化"),
            ("realm_links.graph_subtitle", "上方链接行的空间布局视图。"),
            (
                "realm_links.graph_placeholder",
                "交互式图形可视化开发中 — 暂请使用上方列表。",
            ),
            ("realm_links.kind.governed_by", "受治理于"),
            ("realm_links.kind.discoverable_from", "可被发现于"),
            ("realm_links.kind.mirror_of", "镜像自"),
            // T6.2 §4 — Push route inspector。
            ("nav.push_routes", "推送路由"),
            ("push_routes.title", "推送路由"),
            (
                "push_routes.subtitle",
                "按 principal 聚合的 cx.device.push_route cell。push_target_id 为敏感字段，默认折叠，必须显式展开。",
            ),
            ("push_routes.search", "按 principal id 过滤"),
            ("push_routes.principal", "Principal"),
            ("push_routes.device", "设备"),
            ("push_routes.transport", "传输"),
            ("push_routes.cell_subject", "Cell subject"),
            ("push_routes.status", "状态"),
            ("push_routes.last_rotation", "最近轮换"),
            ("push_routes.target", "Push target id"),
            ("push_routes.empty", "该 principal 暂无推送路由。"),
            ("push_routes.reveal", "展开"),
            ("push_routes.hide", "隐藏"),
            // T6.2 §5 — Capability 约束编辑。
            ("capabilities.edit_title", "编辑能力"),
            ("capabilities.fields_write_allow", "可写字段允许清单"),
            ("capabilities.facets_allow", "Facet 允许清单"),
            ("capabilities.approval_required", "需要审批"),
            ("capabilities.csv_hint", "用逗号分隔"),
            ("capabilities.edit_ok", "已更新能力"),
            ("capabilities.edit_fail", "更新能力失败"),
            // ── 第 24 轮 侧边栏 ──
            ("nav.coauth_capabilities", "Authz 权限"),
            ("nav.spaces_admin", "Space（管理）"),
            ("nav.moderation_reports", "审核举报"),
            // ── 第 24 轮 通用 UI ──
            ("common.refresh", "刷新"),
            ("common.previous", "上一页"),
            ("common.next", "下一页"),
            ("common.confirm", "确认"),
            // ── 第 24 轮 B5：soland authz 权限授予 ──
            ("authz_caps.title", "Authz 权限授予"),
            (
                "authz_caps.subtitle",
                "当前管理员作用域可见的 soland authz 权限授予。可按 holder / peer / scope 过滤；撤销将发布 cx.cell.authz.capability.revoke Move。",
            ),
            ("authz_caps.holder", "Holder DID"),
            ("authz_caps.peer", "Peer DID"),
            ("authz_caps.scope", "Scope"),
            ("authz_caps.status", "状态"),
            ("authz_caps.granted_at", "授予时间"),
            ("authz_caps.expires_at", "过期时间"),
            ("authz_caps.revoke", "撤销"),
            ("authz_caps.filter_holder", "按 Holder DID 过滤"),
            ("authz_caps.filter_peer", "按 Peer DID 过滤"),
            ("authz_caps.filter_scope", "按 Scope 过滤"),
            ("authz_caps.empty_title", "无权限授予"),
            (
                "authz_caps.empty_subtitle",
                "soland 报告没有匹配当前过滤条件的权限授予。",
            ),
            ("authz_caps.revoke_confirm_title", "撤销权限授予？"),
            (
                "authz_caps.revoke_confirm_body",
                "在 authz cell 上发布 revoke Move。依赖此权限的现有会话将在下一次 anchor view 时重新检查。该操作记入审计。继续？",
            ),
            // ── 第 24 轮 B6：账号设备 ──
            ("coauth_devices.title", "账号设备"),
            (
                "coauth_devices.subtitle",
                "注册到该账号的设备列表。撤销操作将级联撤销 soland 端关联的会话授权。",
            ),
            ("coauth_devices.device_id", "设备 ID"),
            ("coauth_devices.display_name", "显示名"),
            ("coauth_devices.status", "状态"),
            ("coauth_devices.last_seen", "最后在线"),
            ("coauth_devices.linked_sessions", "关联会话"),
            ("coauth_devices.revoke", "撤销"),
            ("coauth_devices.empty_title", "尚未注册设备"),
            (
                "coauth_devices.empty_subtitle",
                "coauth 尚未为该账号注册任何设备。",
            ),
            ("coauth_devices.revoke_confirm_title", "撤销设备？"),
            (
                "coauth_devices.revoke_confirm_body",
                "在 coauth 中将设备标记为已撤销，并级联撤销 soland 端绑定到该设备的会话授权。该操作记入审计。继续？",
            ),
            // ── 第 24 轮 D1：Space 管理列表 ──
            ("spaces_admin.title", "Space（管理）"),
            (
                "spaces_admin.subtitle",
                "当前管理员作用域可见的 Space 列表，由 soland 管理 describe 接口提供。",
            ),
            ("spaces_admin.search_placeholder", "按 ID 或名称搜索"),
            ("spaces_admin.id", "ID"),
            ("spaces_admin.name", "名称"),
            ("spaces_admin.members", "成员数"),
            ("spaces_admin.health", "健康度"),
            ("spaces_admin.created_at", "创建时间"),
            ("spaces_admin.open_detail", "打开详情"),
            ("spaces_admin.open_hierarchy", "层级"),
            ("spaces_admin.empty_title", "无 Space"),
            (
                "spaces_admin.empty_subtitle",
                "soland 报告该管理员作用域下没有匹配过滤条件的 Space。",
            ),
            // ── 第 24 轮 D2：Space 层级视图 ──
            ("space_hierarchy.title", "层级"),
            ("space_hierarchy.subtitle", "该 Space 的直接父级与子级。"),
            ("space_hierarchy.parent_label", "父级"),
            ("space_hierarchy.this_label", "本"),
            ("space_hierarchy.child_label", "子级"),
            ("space_hierarchy.children_label", "子级列表"),
            ("space_hierarchy.empty_title", "无关联"),
            (
                "space_hierarchy.empty_subtitle",
                "该 Space 没有父级也没有直接子级。",
            ),
            // ── 第 24 轮 D5：审核举报 ──
            ("moderation_reports.title", "审核举报"),
            (
                "moderation_reports.subtitle",
                "等待管理员决策的待审核举报列表。Resolve 表示已处理结案；Dismiss 表示审核后判定无需处理。",
            ),
            ("moderation_reports.id", "举报 ID"),
            ("moderation_reports.reporter", "举报人"),
            ("moderation_reports.target", "对象"),
            ("moderation_reports.space", "Space"),
            ("moderation_reports.reason", "原因"),
            ("moderation_reports.status", "状态"),
            ("moderation_reports.created_at", "创建时间"),
            ("moderation_reports.resolve", "处理"),
            ("moderation_reports.dismiss", "驳回"),
            ("moderation_reports.filter_open", "待办"),
            ("moderation_reports.filter_resolved", "已处理"),
            ("moderation_reports.filter_dismissed", "已驳回"),
            ("moderation_reports.filter_all", "全部"),
            ("moderation_reports.empty_title", "无审核举报"),
            (
                "moderation_reports.empty_subtitle",
                "soland 报告当前过滤条件下没有审核举报。",
            ),
            ("moderation_reports.resolve_confirm_title", "处理该举报？"),
            (
                "moderation_reports.resolve_confirm_body",
                "在该举报上记录管理员 Resolve 决策。该操作记入审计。继续？",
            ),
            ("moderation_reports.resolve_confirm_btn", "确认处理"),
            ("moderation_reports.dismiss_confirm_title", "驳回该举报？"),
            (
                "moderation_reports.dismiss_confirm_body",
                "在该举报上记录管理员 Dismiss 决策（无需进一步处理）。该操作记入审计。继续？",
            ),
            ("moderation_reports.dismiss_confirm_btn", "确认驳回"),
            ("nav.applets_admin", "Applet（管理）"),
            ("nav.agents_admin", "Agent（管理）"),
            ("nav.directory_admin", "目录（管理）"),
            ("nav.federation_status", "联邦状态"),
            ("nav.policy_editor", "策略编辑"),
            ("nav.starid_resolver", "Starid 解析器"),
            ("starid_resolver.title", "Starid 解析器"),
            (
                "starid_resolver.subtitle",
                "上游 did:webvh 写入端的只读视图——头版本、见证数量与最近活跃时间。",
            ),
            ("starid_resolver.card_title", "解析器状态"),
            (
                "starid_resolver.card_subtitle",
                "/api/v1/identity/describe 报告的聚合状态。",
            ),
            ("starid_resolver.service_did", "服务 DID"),
            ("starid_resolver.registry_mode", "注册模式"),
            ("starid_resolver.protocol_version", "协议版本"),
            ("starid_resolver.head_version_id", "头 version_id"),
            ("starid_resolver.witness_count", "见证数量"),
            ("starid_resolver.freshness", "最近活跃"),
            ("starid_resolver.methods_label", "支持的方法"),
            ("starid_resolver.profiles_label", "配置档"),
            ("starid_resolver.status_healthy", "健康"),
            ("starid_resolver.status_pending", "等待见证"),
            ("starid_resolver.status_idle", "空闲"),
            ("starid_resolver.not_configured_badge", "未配置"),
            (
                "starid_resolver.not_configured_hint",
                "在部署配置中设置 starid_public_url 以启用此面板。",
            ),
            (
                "starid_resolver.not_configured_body",
                "本部署未配置 Starid。sodmin 管理端从部署配置读取上游解析器 URL——一旦配置完成，本面板将自动镜像写入端的状态。",
            ),
            (
                "starid_resolver.docs_link",
                "查看 Starid 解析器文档 \u{2192}",
            ),
            ("federation_status.title", "联邦状态"),
            (
                "federation_status.subtitle",
                "按 Space 列出联邦节点、最近 anchor 拉取时间及出站复制队列深度。",
            ),
            ("federation_status.generated_at", "生成时间"),
            ("federation_status.space_id", "Space"),
            ("federation_status.peer", "对端"),
            ("federation_status.health", "健康度"),
            (
                "federation_status.last_anchor_pulled_at",
                "最近 anchor 拉取",
            ),
            ("federation_status.last_pushed_at", "最近推送"),
            ("federation_status.outbound_queue_depth", "出站队列"),
            ("federation_status.empty_title", "无联邦节点"),
            (
                "federation_status.empty_subtitle",
                "soland 报告本范围内无联邦节点，或联邦状态接口尚未部署。",
            ),
            ("policy_editor.title", "Space 策略编辑"),
            (
                "policy_editor.subtitle",
                "编辑 cx.component.space.policy.v1 组件。提交将构造 cas-register Move。",
            ),
            ("policy_editor.history_visibility", "历史可见性"),
            ("policy_editor.join_rule", "加入规则"),
            ("policy_editor.guest_access", "访客访问"),
            ("policy_editor.federate", "联邦"),
            ("policy_editor.encryption_algorithm", "加密算法"),
            ("policy_editor.note", "提交备注"),
            ("policy_editor.submit", "提交策略更新"),
            ("policy_editor.confirm_title", "提交策略更新？"),
            (
                "policy_editor.confirm_body",
                "构造并签名 cx.component.space.policy.v1 上的 cas-register Move。该操作记入审计。继续？",
            ),
            ("policy_editor.confirm_btn", "确认提交"),
            ("applets_admin.title", "Applet（管理）"),
            (
                "applets_admin.subtitle",
                "等待 soland 端审批/撤销的 applet 注册。",
            ),
            ("applets_admin.id", "ID"),
            ("applets_admin.name", "名称"),
            ("applets_admin.owner_did", "拥有者 DID"),
            ("applets_admin.status", "状态"),
            ("applets_admin.registered_at", "注册时间"),
            ("applets_admin.empty_title", "无 applet 注册"),
            (
                "applets_admin.empty_subtitle",
                "soland 报告无 applet 注册，或该接口尚未部署。",
            ),
            ("applets_admin.approve_confirm_title", "批准 applet？"),
            (
                "applets_admin.approve_confirm_body",
                "将 applet 注册标记为已批准。该操作记入审计。",
            ),
            ("applets_admin.revoke_confirm_title", "撤销 applet？"),
            (
                "applets_admin.revoke_confirm_body",
                "将 applet 注册标记为已撤销。该操作记入审计。",
            ),
            ("applets_admin.suspend_confirm_title", "暂停 applet？"),
            (
                "applets_admin.suspend_confirm_body",
                "暂时停用该 applet，但保留其身份。重新批准即可恢复。该操作记入审计。",
            ),
            ("agents_admin.title", "Agent（管理）"),
            (
                "agents_admin.subtitle",
                "等待 soland 端审批/撤销的 agent 注册。",
            ),
            ("agents_admin.id", "ID"),
            ("agents_admin.name", "名称"),
            ("agents_admin.owner_did", "拥有者 DID"),
            ("agents_admin.status", "状态"),
            ("agents_admin.registered_at", "注册时间"),
            ("agents_admin.empty_title", "无 agent 注册"),
            (
                "agents_admin.empty_subtitle",
                "soland 报告无 agent 注册，或该接口尚未部署。",
            ),
            ("agents_admin.approve_confirm_title", "批准 agent？"),
            (
                "agents_admin.approve_confirm_body",
                "将 agent 注册标记为已批准。该操作记入审计。",
            ),
            ("agents_admin.revoke_confirm_title", "撤销 agent？"),
            (
                "agents_admin.revoke_confirm_body",
                "将 agent 注册标记为已撤销。该操作记入审计。",
            ),
            ("agents_admin.suspend_confirm_title", "暂停 agent？"),
            (
                "agents_admin.suspend_confirm_body",
                "暂时停用该 agent，但保留其身份。重新批准即可恢复。该操作记入审计。",
            ),
            ("directory_admin.title", "目录（管理）"),
            (
                "directory_admin.subtitle",
                "等待 soland 端审批/撤销的目录条目。",
            ),
            ("directory_admin.entry_id", "条目 ID"),
            ("directory_admin.kind", "类型"),
            ("directory_admin.label", "标签"),
            ("directory_admin.owner_did", "拥有者 DID"),
            ("directory_admin.status", "状态"),
            ("directory_admin.published_at", "发布时间"),
            ("directory_admin.empty_title", "无目录条目"),
            (
                "directory_admin.empty_subtitle",
                "soland 报告无目录条目，或该接口尚未部署。",
            ),
            ("directory_admin.approve_confirm_title", "批准条目？"),
            (
                "directory_admin.approve_confirm_body",
                "将目录条目标记为已批准（公开列表中可见）。该操作记入审计。",
            ),
            ("directory_admin.revoke_confirm_title", "撤销条目？"),
            (
                "directory_admin.revoke_confirm_body",
                "将目录条目标记为已撤销（从公开列表移除）。该操作记入审计。",
            ),
            ("directory_admin.reject_confirm_title", "驳回条目？"),
            (
                "directory_admin.reject_confirm_body",
                "驳回该目录条目——不会被公开列出。该操作记入审计。",
            ),
            // ── CXP-0007 Circle 管理（P3A.3 / P3A.6 / P3A.8） ─────────
            ("nav.circles", "Circle"),
            ("nav.section_circles", "Circle"),
            ("circle.list_title", "Circle 列表"),
            (
                "circle.list_description",
                "Realm 内的加密子边界。每个 Circle 携带自己的 MLS 组，其成员必须是父 Realm 成员的严格子集。",
            ),
            ("circle.create", "创建 Circle"),
            (
                "circle.create_description",
                "在已存在的 Realm 中创建新的 Circle。父 Realm 在创建后不可修改。",
            ),
            ("circle.id", "Circle ID"),
            ("circle.title", "标题"),
            ("circle.title_placeholder", "信任与安全"),
            ("circle.summary", "摘要"),
            ("circle.realm_id", "Realm"),
            (
                "circle.realm_id_hint",
                "父 Realm 的 ck:realm:... 标识符。必填。",
            ),
            ("circle.filter_realm", "按 Realm 过滤"),
            (
                "circle.filter_realm_hint",
                "Circle 始终归属单个 Realm。粘贴 ck:realm:... 以加载该 Realm 下的所有 Circle。",
            ),
            ("circle.filter_apply", "应用"),
            ("circle.directory_visibility", "目录可见性"),
            ("circle.join_rule", "加入规则"),
            ("circle.history_visibility", "历史可见性"),
            ("circle.encryption_profile", "加密配置"),
            ("circle.mls_group_ref", "MLS 组引用"),
            ("circle.member_count", "成员数"),
            ("circle.members_total", "成员总数："),
            ("circle.created_at", "创建时间"),
            ("circle.created_by", "创建者"),
            ("circle.updated_at", "更新时间"),
            ("circle.state", "状态"),
            ("circle.state_active", "活跃"),
            ("circle.state_archived", "已归档"),
            ("circle.state_tombstoned", "已墓碑"),
            ("circle.state_unknown", "未知"),
            ("circle.overview", "概览"),
            ("circle.actions", "操作"),
            ("circle.manage_members", "管理成员"),
            ("circle.manage_scope", "管理域（MLS scope）"),
            (
                "circle.scope_description",
                "轮换 Circle 绑定的 MLS 组。即使本次没有成员变更，回执也会进入审计日志。",
            ),
            (
                "circle.scope_hint",
                "完整的 MLS 密钥轮换将随 soland P2A.4 上线。目前仅发送 cx.circle.update Move 以触发授权挂钩。",
            ),
            ("circle.rotate_scope", "轮换 MLS 域"),
            ("circle.archive", "归档"),
            ("circle.tombstone", "墓碑（不可逆）"),
            ("circle.add_member", "添加成员"),
            ("circle.remove_member", "移除"),
            ("circle.actor_id", "Actor ID"),
            ("circle.member_state", "状态"),
            ("circle.empty_members", "暂无成员。"),
            ("circle.empty_title", "此 Realm 暂无 Circle"),
            (
                "circle.empty_description",
                "点击「创建 Circle」以添加第一个加密子边界。",
            ),
            ("circle.empty_no_realm_title", "请选择 Realm"),
            (
                "circle.empty_no_realm_description",
                "Circle 始终归属单个 Realm。请在上方输入 ck:realm:... 以加载该 Realm 下的 Circle。",
            ),
            (
                "circle.subset_hint",
                "成员必须已属于父 Realm。soland 会以 `circle_member_must_be_realm_member` 拒绝越界添加。",
            ),
            ("circle.error_required", "realm_id 与 title 均为必填。"),
            ("circle.error_actor_required", "actor_id 必填。"),
            ("circle.created_toast", "Circle 已创建。"),
            ("circle.archived_toast", "Circle 已归档。"),
            ("circle.tombstoned_toast", "Circle 已墓碑（终态）。"),
            ("circle.scope_rotated_toast", "已确认 Circle 域轮换。"),
            ("circle.member_added_toast", "成员已加入 Circle。"),
            ("circle.member_removed_toast", "成员已从 Circle 移除。"),
            ("circle.confirm_archive_title", "归档此 Circle？"),
            (
                "circle.confirm_archive_body",
                "归档后将拒绝所有成员变更与常规写入。仍可进行域轮换。软删窗口内可恢复。",
            ),
            ("circle.confirm_tombstone_title", "墓碑此 Circle？"),
            (
                "circle.confirm_tombstone_body",
                "墓碑为终态：不再接受任何管理操作，成员状态永久冻结。CXP-0007 会发出擦除回执。",
            ),
            // D.1 — plural `circles.*` namespace, paired with the
            // singular `circle.*` keys above.
            ("circles.title", "Circle"),
            ("circles.create", "创建 Circle"),
            ("circles.member_limit", "成员必须已属于父 Realm。"),
            // B.7 — Export CSV button label.
            ("common.export_csv", "导出 CSV"),
            ("common.enabled", "已启用"),
            ("common.disabled", "已禁用"),
            // P3A.6 — Principal Control vs Collaboration Realm
            ("realm.principal_control", "主体控制（Principal Control）"),
            ("realm.collaboration", "协作（Collaboration）"),
            ("realm.classification", "Realm 类别"),
            (
                "realm.classification_hint",
                "主体控制 Realm 承担 DID 颁发与恢复；Collaboration Realm 仅作内容范围。CXP-0007 规定该分类在创建后不可变更。",
            ),
            // P3A.5 — audit effective_scope + Circle 事件筛选
            ("audit.effective_scope", "有效范围"),
            ("audit.scope_realm", "Realm"),
            ("audit.scope_circle", "Circle"),
            ("audit.scope_jump", "跳转到该范围"),
            ("audit.filter_event_kind", "事件类型"),
            ("audit.filter_event_kind_circle_create", "cx.circle.create"),
            ("audit.filter_event_kind_circle_update", "cx.circle.update"),
            (
                "audit.filter_event_kind_circle_archive",
                "cx.circle.archive",
            ),
            (
                "audit.filter_event_kind_circle_tombstone",
                "cx.circle.tombstone",
            ),
            (
                "audit.filter_event_kind_circle_member_state",
                "cx.circle.member.state",
            ),
            (
                "audit.filter_event_kind_circle_capability_grant",
                "cx.circle.capability.grant",
            ),
            (
                "audit.filter_event_kind_circle_capability_revoke",
                "cx.circle.capability.revoke",
            ),
            // P3A.4 — capability 编辑器（cx.circle.*）标签
            ("capability.cx_circle_create", "cx.circle.create"),
            ("capability.cx_circle_manage", "cx.circle.manage"),
            ("capability.cx_circle_member_add", "cx.circle.member.add"),
            (
                "capability.cx_circle_member_manage",
                "cx.circle.member.manage",
            ),
            (
                "capability.cx_circle_member_add_others",
                "cx.circle.member.add.others",
            ),
            ("capability.cx_circle_audit", "cx.circle.audit"),
            ("capability.cx_circle_section", "Circle 能力（CXP-0007）"),
            ("capability.allowed_circle_ids", "受限 Circle ID 列表"),
            (
                "capability.allowed_circle_ids_hint",
                "逗号分隔的 ck:circle:... 标识符。cx.circle.manage、cx.circle.member.manage、cx.circle.member.add.others 必填。",
            ),
            // P3A.8 — CXP-0007 错误码（来自 reducer）
            (
                "error.circle_realm_mismatch",
                "Circle 与 Realm 不匹配——请求的操作目标 Realm 与 Circle 所绑定的 Realm 不一致。",
            ),
            (
                "error.circle_member_must_be_realm_member",
                "成员必须已属于父 Realm。",
            ),
            (
                "error.circle_not_active",
                "Circle 处于非活跃状态——已归档或已墓碑的 Circle 拒绝此操作。",
            ),
            (
                "error.circle_already_terminal",
                "Circle 已处于终态（已归档或已墓碑）。",
            ),
            (
                "error.circle_capability_denied",
                "缺少所需的 cx.circle.* 能力，或能力仅授予不同的 Circle。",
            ),
            (
                "error.circle_scope_rotation_in_progress",
                "另一次域轮换正在进行中。",
            ),
            // P5 — /agents/personal 页面中文短串。
            ("agents.personal.title", "个人 Agent"),
            (
                "agents.personal.subtitle",
                "控制者自管原生 Agent（CXP-0008）",
            ),
            ("agents.personal.provision", "新建 Agent"),
            ("agents.personal.deactivate", "停用"),
            ("agents.personal.pause", "暂停"),
            ("agents.personal.resume", "恢复"),
            ("agents.personal.rotate_key", "轮换密钥"),
            ("agents.personal.no_agents", "暂无个人 Agent。"),
            ("agents.personal.controller_did", "控制者 DID"),
            ("agents.personal.display_name", "显示名称（可选）"),
            ("agents.personal.step_label", "第 {step}/3 步"),
            ("agents.personal.identity", "身份"),
            ("agents.personal.agent_keys", "Agent 密钥"),
            ("agents.personal.capability_grants", "授权能力"),
            ("agents.personal.required_capability", "所需能力"),
            // R3 — Agent FSM 列表（UI-1）。
            ("agents.title", "Agent"),
            (
                "agents.subtitle",
                "原生与个人 Agent —— active / paused / deactivated 状态机。",
            ),
            ("agents.id", "Agent ID"),
            ("agents.name", "名称"),
            ("agents.owner_id", "拥有者"),
            ("agents.agent_type", "类型"),
            ("agents.status", "状态"),
            ("agents.last_active_at", "上次活跃"),
            ("agents.no_agents", "暂无 Agent。"),
            ("agents.pause", "暂停"),
            ("agents.resume", "恢复"),
            ("agents.rotate_key", "轮换密钥"),
            ("agents.grants", "授权"),
            ("agents.deactivate", "停用"),
            ("agents.deactivate_title", "停用该 Agent？"),
            (
                "agents.deactivate_body",
                "此操作将吊销 Agent 的密钥、能力授权与运行时端点。请输入 DEACTIVATE 以确认。",
            ),
            // R3 — Agent 草稿/动作生命周期（UI-2 占位）。
            ("agents.lifecycle.title", "Agent 草稿 / 动作生命周期"),
            (
                "agents.lifecycle.subtitle",
                "Drafts、action_request、action_approve、action_reject。R3.1 接入。",
            ),
            ("agents.lifecycle.placeholder", "草稿 / 动作数据尚未接入。"),
            // R3 — Realm media_service 编辑器（UI-3）。
            ("media_service.title", "Realm 媒体服务"),
            (
                "media_service.subtitle",
                "Foci 列表 —— 选择一个或多个媒体服务（livekit、mediasoup、janus、contrix-native、moq-relay）。",
            ),
            ("media_service.foci_label", "Foci"),
            ("media_service.empty", "当前 Realm 未配置 foci。"),
            ("media_service.add", "新增 focus"),
            ("media_service.focus_id", "id"),
            ("media_service.focus_type", "type"),
            ("media_service.focus_connect_url", "connect_url"),
            ("media_service.focus_service_did", "service_did"),
            ("media_service.focus_regions", "regions（逗号分隔）"),
            ("media_service.remove", "移除"),
            ("media_service.save", "保存 media_service"),
            ("media_service.save_ok", "media_service 已保存"),
            ("media_service.save_fail", "保存 media_service 失败"),
            (
                "media_service.legacy_banner_title",
                "检测到旧版单端点 media_service",
            ),
            (
                "media_service.legacy_banner_body",
                "sfu_endpoint 字段已被 foci[] 取代。请将旧端点写入首个 focus，扩展其它服务后保存以完成迁移。",
            ),
            // R3 — 恢复策略视图（UI-4 占位）。
            ("recovery.title", "恢复策略"),
            (
                "recovery.subtitle",
                "列出、查看、轮换 cx.coauth.recovery.* 策略；receipt 携带 proof_summary[]。",
            ),
            ("recovery.list_title", "策略"),
            ("recovery.policy_id", "策略 ID"),
            ("recovery.policy_type", "类型"),
            ("recovery.policy_status", "状态"),
            ("recovery.policy_actions", "操作"),
            ("recovery.inspect", "查看"),
            ("recovery.rotate", "轮换"),
            ("recovery.receipts_title", "恢复回执"),
            (
                "recovery.receipts_subtitle",
                "每条回执的 proof_summary[] 列出 coauth 恢复流程接受的证明项。",
            ),
            (
                "recovery.placeholder",
                "恢复数据 API 在 R3.1 接入 —— 当前为占位界面。",
            ),
            // R3 — Handle homograph 内联提示（UI-5）。
            (
                "error.handle_homograph_forbidden",
                "该 Handle 会被服务端拒绝为同形异义体（handle_homograph_forbidden）。",
            ),
            (
                "error.handle_homograph_out_of_range",
                "Handle 本地段长度必须在 1..=128 字节之间。",
            ),
            (
                "error.handle_homograph_zero_width",
                "Handle 包含零宽 / 方向控制码点 —— 已拒绝。",
            ),
            (
                "error.handle_homograph_script_mixed",
                "Handle 混用了 Latin 与非 Latin 文字 —— 视为同形异义体已拒绝。",
            ),
            (
                "error.handle_homograph_confusable",
                "Handle 含有可与 ASCII 字母混淆的码点 —— 已拒绝。",
            ),
            // R3 — Server profile 开关说明（UI-6）。
            (
                "server_status.profile.media_service_binding",
                "media_service_binding.v1 —— Realm 级 foci 绑定 profile。",
            ),
            (
                "server_status.profile.accountable_principals_strict_reject",
                "accountable_principals.strict_reject.v1 —— 缺失 accountability grant 时 fail-closed。",
            ),
            (
                "server_status.profile.key_backup_memory_hard",
                "key_backup.memory_hard.v1 —— Argon2id 内存密集型 KDF。",
            ),
            // R3 — 新增 wire 错误码（UI-7）。
            (
                "error.pairing_request_expired",
                "配对请求已在批准前过期，请重新发起。",
            ),
            ("error.proof_invalid", "提交的证明未通过校验。"),
            ("error.agent_paused", "Agent 处于暂停状态，请先恢复后重试。"),
            ("error.agent_deactivated", "Agent 已停用，无法接受新工作。"),
            (
                "error.accountability_grant_missing",
                "该操作需要控制者签发的 accountability_grant。",
            ),
            (
                "error.participant_binding_invalid",
                "Participant binding 与 Realm 允许的 recipient service 不匹配。",
            ),
            (
                "error.legacy_single_endpoint_media_service",
                "该 Realm 仍在使用旧版 sfu_endpoint —— 请迁移至 foci[]。",
            ),
            // P5 — /key-backup 页面中文短串。
            ("key_backup.title", "密钥备份"),
            ("key_backup.subtitle", "服务端交叉签名密钥备份"),
            ("key_backup.no_backup", "服务器上未找到密钥备份。"),
            ("key_backup.backup_version", "备份版本"),
            ("key_backup.algorithm", "算法"),
            ("key_backup.created", "创建时间"),
            ("key_backup.delete", "删除备份"),
            ("key_backup.delete_confirm", "确认删除该备份版本？"),
            ("key_backup.refresh", "刷新"),
            // R3.1 (HDLREN-1) — handle 规范化错误信息。
            ("error.handle_empty", "句柄为空（已去除空白）。"),
            (
                "error.handle_malformed",
                "句柄无法解析为 <localpart>:<domain>，请使用规范的 wire 格式。",
            ),
            // R3.1 (MID-1..3, ROST-1..2) — Realm 成员 + 身份诊断。
            ("spaces.identity", "身份"),
            ("spaces.membership", "成员状态"),
            (
                "spaces.members_subtitle",
                "句柄和显示名称源自当前有效的 MemberIdentity，而非旧版同步成员字段。",
            ),
            ("spaces.identity_pending", "身份待解密"),
            ("spaces.members_showing", "显示"),
            ("spaces.members_of_many", "项（共更多）"),
            ("spaces.members_load_more", "加载更多"),
            ("spaces.open_identity_audit", "打开身份审计"),
            ("realm_identity_audit.title", "身份审计"),
            (
                "realm_identity_audit.subtitle",
                "每位 actor 当前有效的 cx.member.identity.update 事件 id 及 member_display_state_digest 投影。",
            ),
            ("realm_identity_audit.stub_title", "数据接入待完成"),
            (
                "realm_identity_audit.stub_body",
                "在 yougen MLS 解密管线（MID-4）就绪后，此诊断页将开始展示真实数据。今天先渲染空骨架以便从 Realm 详情卡片到达本页。",
            ),
            ("realm_identity_audit.actors_title", "Actor 列表"),
            (
                "realm_identity_audit.actors_subtitle",
                "本 Realm 中每个 actor 一行。本地投影与 roster 不一致时 cache_drift 会标红。",
            ),
            ("realm_identity_audit.col_actor", "Actor DID"),
            ("realm_identity_audit.col_primary_handle", "首选句柄"),
            ("realm_identity_audit.col_display_name", "显示名称"),
            ("realm_identity_audit.col_event_ids", "有效事件 id"),
            (
                "realm_identity_audit.col_state_digest",
                "member_display_state_digest",
            ),
            ("realm_identity_audit.col_cache", "缓存"),
            ("realm_identity_audit.cache_drift", "偏差"),
            ("realm_identity_audit.cache_ok", "正常"),
            ("realm_identity_audit.empty", "暂未观察到任何 actor。"),
            // R3.2 (UI-SOD-4) — Subject → Handles 目录页。
            ("nav.handles_by_subject", "Subject → Handles"),
            ("handles_by_subject.title", "Subject → Handles"),
            (
                "handles_by_subject.subtitle",
                "按 holder/principal DID 反查当前可见的已签名 handle claim（cx.directory.list_handles_for_subject）。",
            ),
            ("handles_by_subject.lookup_title", "查询主体"),
            (
                "handles_by_subject.lookup_body",
                "输入 holder/principal DID。MemberIdentity 不再携带 handle —— handle 来自已签名的 cx.schema.handle_claim.v1 凭证。",
            ),
            ("handles_by_subject.subject_id", "主体 ID"),
            ("handles_by_subject.lookup", "查询"),
            (
                "handles_by_subject.enter_did",
                "在上方输入主体 DID 以列出其可见的 handle claim。",
            ),
            ("handles_by_subject.results_title", "可见的 handle claim"),
            (
                "handles_by_subject.primary_handle",
                "Primary handle（§3.2.1）",
            ),
            ("handles_by_subject.primary_badge", "primary"),
            ("handles_by_subject.col_handle", "Handle"),
            ("handles_by_subject.col_issuer", "签发者 DID"),
            ("handles_by_subject.col_binding", "绑定状态"),
            ("handles_by_subject.col_expires", "过期时间"),
            ("handles_by_subject.col_why", "为什么？"),
            ("handles_by_subject.why", "为什么我能看到它？"),
            ("handles_by_subject.why_issuer", "签发者"),
            ("handles_by_subject.why_binding", "binding_state"),
            ("handles_by_subject.why_created", "created_at"),
            (
                "handles_by_subject.no_claims",
                "当前 context 下该主体没有可见的 handle claim。",
            ),
            (
                "handles_by_subject.has_more",
                "还有更多 claim —— 收窄 context 以缩小结果集。",
            ),
            ("handles_by_subject.binding_verified", "verified"),
            ("handles_by_subject.binding_pending", "pending"),
            ("handles_by_subject.binding_revoked", "revoked"),
            ("handles_by_subject.binding_expired", "expired"),
            // R3.2 (UI-SOD-5) — handle changed since 提示。
            ("handle_change.changed_since", "handle 已变更"),
            ("handle_change.now", "当前"),
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

pub fn set_language(lang: Language) {
    crate::utils::storage::set_item("language", lang.code());
    *CURRENT_LANG.write() = lang;
    apply_document_language(lang);
}

pub fn current_language() -> Language {
    *CURRENT_LANG.read()
}

pub fn sync_document_language() {
    let lang = crate::utils::storage::get_item("language")
        .and_then(|s| Language::from_code(&s))
        .unwrap_or(Language::En);
    apply_document_language(lang);
}

fn apply_document_language(lang: Language) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(root) = document.document_element() else {
        return;
    };
    let _ = root.set_attribute("lang", lang.code());
}
