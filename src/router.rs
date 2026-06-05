use dioxus::prelude::*;

use crate::api::auth;
use crate::components::layout::AppLayout;
use crate::pages;
use crate::utils::i18n::t;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login")]
    LoginPage {},

    #[route("/oauth/callback?:code&:state&:error&:error_description")]
    OAuthCallback {
        code: Option<String>,
        state: Option<String>,
        error: Option<String>,
        error_description: Option<String>,
    },

    #[layout(AuthenticatedLayout)]
        #[route("/")]
        Dashboard {},

        #[route("/actors")]
        ActorList {},
        #[route("/actors/create")]
        ActorCreate {},
        #[route("/actors/:actor_id")]
        ActorShow { actor_id: String },

        #[route("/realms")]
        RealmList {},
        #[route("/realms/create")]
        RealmCreate {},
        #[route("/realms/:realm_id")]
        RealmShow { realm_id: String },
        #[route("/realms/:realm_id/anchorer")]
        RealmAnchorer { realm_id: String },
        #[route("/realms/:realm_id/anchor-dag")]
        RealmAnchorDag { realm_id: String },
        #[route("/realms/:realm_id/consent")]
        RealmConsent { realm_id: String },
        #[route("/realms/:realm_id/covered-frontier")]
        RealmCoveredFrontier { realm_id: String },
        #[route("/realms/:realm_id/signing-keys")]
        RealmSigningKeys { realm_id: String },
        #[route("/realms/:realm_id/multisig")]
        RealmMultiSig { realm_id: String },
        #[route("/realms/:realm_id/federation-status")]
        RealmFederationStatus { realm_id: String },
        #[route("/realms/:realm_id/policy-editor")]
        RealmPolicyEditor { realm_id: String },

        #[route("/spaces")]
        SpaceList {},
        #[route("/spaces/:space_id")]
        SpaceShow { space_id: String },

        #[route("/anchor/bottom")]
        AnchorBottom {},

        #[route("/components")]
        ComponentsRegistry {},

        #[route("/media")]
        MediaList {},

        #[route("/reports")]
        ReportList {},
        #[route("/reports/:report_id")]
        ReportShow { report_id: String },

        #[route("/moderation/reports")]
        ModerationReports {},
        // Round R2/R3 T06 — moderation appeal admin
        // (`ck.moderation.appeal.*`).
        #[route("/moderation/appeals")]
        ModerationAppeals {},

        #[route("/federation")]
        FederationList {},
        #[route("/federation/:domain")]
        FederationShow { domain: String },

        #[route("/devices")]
        DeviceList {},

        #[route("/capabilities")]
        CapabilityList {},

        // CKP-0007 Circle admin (P3A.3). Circles are encrypted
        // sub-boundaries inside a Realm; full CRUD + member +
        // scope-rotation surface lives under `/circles/*`.
        #[route("/circles")]
        CircleList {},
        #[route("/circles/new")]
        CircleCreate {},
        #[route("/circles/:circle_id")]
        CircleShow { circle_id: String },
        #[route("/circles/:circle_id/members")]
        CircleMembers { circle_id: String },
        #[route("/circles/:circle_id/scope")]
        CircleScope { circle_id: String },

        #[route("/handles")]
        HandleList {},
        #[route("/handles/:handle_id")]
        HandleShow { handle_id: String },

        #[route("/push-routes")]
        PushRouteList {},

        #[route("/realms/:realm_id/delivery-binding")]
        RealmDeliveryBinding { realm_id: String },

        // R5.2 — Realm link-graph admin page. Sits next to delivery
        // binding because they share the same Realm-scoped /realms/:id/
        // URL prefix and are conceptually a pair (binding policy +
        // typed boundary edges).
        #[route("/realms/:realm_id/links")]
        RealmLinks { realm_id: String },

        // R3 (UI-3) — Realm media_service.foci[] editor.
        #[route("/realms/:realm_id/media-service")]
        RealmMediaService { realm_id: String },

        // R3.1 (MID-3) — Realm identity audit diagnostic page. Stub
        // view today; full data plumbing lands after yougen MID-4 ships
        // the MLS decrypt pipeline (TODO(R4)).
        #[route("/admin/realms/:realm_id/identity-audit")]
        RealmIdentityAudit { realm_id: String },

        // R3.2 (UI-SOD-4) — Subject → Handles directory page. Operator
        // enters a holder/principal DID; the page calls
        // `ck.find.directory.list_handles_for_subject` and lists the visible
        // signed handle claims + the §3.2.1 primary handle.
        #[route("/admin/handles/by-subject?:subject")]
        HandlesBySubject { subject: Option<String> },

        #[route("/applets")]
        AppletList {},
        #[route("/applets/admin")]
        AppletAdmin {},

        #[route("/agents")]
        AgentList {},
        #[route("/agents/admin")]
        AgentAdmin {},
        // CKP-0008 personal-agent admin (P3-A). List + detail + 3-step
        // provision wizard. Must precede the `/agents/:agent_id` catch
        // so `/agents/personal` does NOT bind agent_id="personal".
        #[route("/agents/personal")]
        PersonalAgentList {},
        #[route("/agents/personal/:agent_id")]
        PersonalAgentShow { agent_id: String },
        #[route("/agents/:agent_id")]
        AgentShow { agent_id: String },

        // B-C key-backup admin surface (P3-B).
        #[route("/key-backup")]
        KeyBackupList {},

        #[route("/directory")]
        DirectoryAdmin {},

        #[route("/audit")]
        AuditLog {},

        #[route("/invite-tokens")]
        InviteTokenList {},

        // Round 4 — 3PID third-party-invite state-machine admin view.
        #[route("/invites/3pid")]
        ThirdPartyInvites {},

        #[route("/policy")]
        PolicyList {},

        #[route("/server-status")]
        ServerStatus {},

        #[route("/hardening")]
        HardeningDashboard {},

        // Round R2/R3 T07 — deactivation 7-domain fanout review.
        #[route("/deactivations/review")]
        DeactivationReview {},
        // Round R2/R3 T07 — realm destroy confirmation + post-anchor
        // fanout + erasure receipt panel.
        #[route("/realms/:realm_id/destroy")]
        RealmDestroy { realm_id: String },
        // Round R2/R3 T08 — deployment-wide trust_domain edit.
        #[route("/server/trust-domain")]
        TrustDomainConfig {},
        // Round R2/R3 T09 — relaxed ephemeral window slider.
        #[route("/server/relaxed-window")]
        RelaxedWindow {},
        // Round R2/R3 T10 — audit attestation evidence upload + review.
        #[route("/audit/attestation")]
        AuditAttestation {},

        #[route("/starid/resolver")]
        StaridResolver {},

        #[route("/coauth/audit-log")]
        CoauthAuditLog {},
        #[route("/coauth/accounts")]
        CoauthAccountList {},
        #[route("/coauth/accounts/:account_id")]
        CoauthAccountShow { account_id: String },
        #[route("/coauth/accounts/:account_id/devices")]
        CoauthAccountDevices { account_id: String },
        #[route("/coauth/capabilities")]
        CoauthCapabilities {},
        #[route("/coauth/oauth2-sessions")]
        CoauthOAuth2Sessions {},
        #[route("/coauth/personal-sessions")]
        CoauthPersonalSessions {},
        #[route("/coauth/upstream-providers")]
        CoauthUpstreamProviders {},
        #[route("/coauth/upstream-links")]
        CoauthUpstreamLinks {},
        #[route("/coauth/registration-tokens")]
        CoauthRegistrationTokens {},
        #[route("/coauth/notification-channels")]
        CoauthNotificationChannels {},
        #[route("/coauth/notification-templates")]
        CoauthNotificationTemplates {},
        #[route("/coauth/connector-health")]
        CoauthConnectorHealth {},
        // R3 (UI-4) — recovery policy admin (stub).
        #[route("/coauth/recovery")]
        CoauthRecovery {},

    #[end_layout]

    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn AuthenticatedLayout() -> Element {
    let nav = use_navigator();

    if !auth::is_authenticated() {
        nav.replace(Route::LoginPage {});
        return rsx! {
            div { "Redirecting..." }
        };
    }

    let cached = auth::cached_is_admin();
    let admin_probe = use_resource(move || async move {
        match auth::verify_admin().await {
            Ok(flag) => Some(flag),
            Err(_) => cached,
        }
    });

    let verdict = match admin_probe.read().as_ref() {
        Some(Some(v)) => Some(*v),
        Some(None) | None => cached,
    };

    match verdict {
        Some(true) => rsx! {
            AppLayout {
                Outlet::<Route> {}
            }
        },
        Some(false) => rsx! {
            pages::not_authorized::NotAuthorizedPage {}
        },
        None => rsx! {
            div { class: "flex min-h-screen items-center justify-center",
                crate::components::ui::loading::Spinner { class: String::new() }
            }
        },
    }
}

#[component]
fn LoginPage() -> Element {
    rsx! {
        pages::login::LoginPage {}
    }
}

#[component]
fn OAuthCallback(
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
) -> Element {
    rsx! {
        pages::oauth_callback::OAuthCallback {
            code: code,
            state: state,
            error: error,
            error_description: error_description,
        }
    }
}

#[component]
fn Dashboard() -> Element {
    rsx! { pages::dashboard::Dashboard {} }
}

#[component]
fn ActorList() -> Element {
    rsx! { pages::actors::list::ActorList {} }
}

#[component]
fn ActorCreate() -> Element {
    rsx! { pages::actors::create::ActorCreate {} }
}

#[component]
fn ActorShow(actor_id: String) -> Element {
    rsx! { pages::actors::show::ActorShow { actor_id } }
}

#[component]
fn RealmList() -> Element {
    rsx! { pages::realms::list::RealmList {} }
}

#[component]
fn RealmCreate() -> Element {
    rsx! { pages::realms::create::RealmCreate {} }
}

#[component]
fn RealmShow(realm_id: String) -> Element {
    rsx! { pages::realms::show::RealmShow { realm_id } }
}

#[component]
fn RealmAnchorer(realm_id: String) -> Element {
    rsx! { pages::realms::anchorer::AnchorerPage { realm_id } }
}

#[component]
fn RealmAnchorDag(realm_id: String) -> Element {
    rsx! { pages::realms::anchor_dag::AnchorDagPage { realm_id } }
}

#[component]
fn AnchorBottom() -> Element {
    rsx! { pages::anchor_bottom::BottomDiagnosticsPage {} }
}

#[component]
fn RealmConsent(realm_id: String) -> Element {
    rsx! { pages::realms::consent::ConsentPage { realm_id } }
}

#[component]
fn RealmCoveredFrontier(realm_id: String) -> Element {
    rsx! { pages::realms::covered_frontier::CoveredFrontierPage { realm_id } }
}

#[component]
fn RealmSigningKeys(realm_id: String) -> Element {
    rsx! { pages::realms::signing_keys::SigningKeysPage { realm_id } }
}

#[component]
fn RealmMultiSig(realm_id: String) -> Element {
    rsx! { pages::realms::multisig::MultiSigPage { realm_id } }
}

#[component]
fn SpaceList() -> Element {
    rsx! { pages::spaces::list::SpaceList {} }
}

#[component]
fn SpaceShow(space_id: String) -> Element {
    rsx! { pages::spaces::show::SpaceShow { space_id } }
}

#[component]
fn ComponentsRegistry() -> Element {
    rsx! { pages::components_registry::ComponentsPage {} }
}

#[component]
fn MediaList() -> Element {
    rsx! { pages::media::MediaList {} }
}

#[component]
fn ReportList() -> Element {
    rsx! { pages::reports::list::ReportList {} }
}

#[component]
fn ReportShow(report_id: String) -> Element {
    rsx! { pages::reports::show::ReportShow { report_id } }
}

#[component]
fn ModerationReports() -> Element {
    rsx! { pages::moderation::reports::ModerationReportsPage {} }
}

#[component]
fn ModerationAppeals() -> Element {
    rsx! { pages::moderation::appeals::ModerationAppealsPage {} }
}

#[component]
fn DeactivationReview() -> Element {
    rsx! { pages::deactivation_review::DeactivationReviewPage {} }
}

#[component]
fn RealmDestroy(realm_id: String) -> Element {
    rsx! { pages::realm_destroy::RealmDestroyPage { realm_id } }
}

#[component]
fn TrustDomainConfig() -> Element {
    rsx! { pages::trust_domain::TrustDomainConfigPage {} }
}

#[component]
fn RelaxedWindow() -> Element {
    rsx! { pages::relaxed_window::RelaxedWindowPage {} }
}

#[component]
fn AuditAttestation() -> Element {
    rsx! { pages::audit_attestation::AuditAttestationPage {} }
}

#[component]
fn FederationList() -> Element {
    rsx! { pages::federation::list::FederationList {} }
}

#[component]
fn FederationShow(domain: String) -> Element {
    rsx! { pages::federation::show::FederationShow { domain } }
}

#[component]
fn DeviceList() -> Element {
    rsx! { pages::devices::DeviceList {} }
}

#[component]
fn CapabilityList() -> Element {
    rsx! { pages::capabilities::CapabilityList {} }
}

#[component]
fn CircleList() -> Element {
    rsx! { pages::circles::list::CircleList {} }
}

#[component]
fn CircleCreate() -> Element {
    rsx! { pages::circles::create::CircleCreate {} }
}

#[component]
fn CircleShow(circle_id: String) -> Element {
    rsx! { pages::circles::show::CircleShow { circle_id } }
}

#[component]
fn CircleMembers(circle_id: String) -> Element {
    rsx! { pages::circles::members::CircleMembers { circle_id } }
}

#[component]
fn CircleScope(circle_id: String) -> Element {
    rsx! { pages::circles::scope::CircleScope { circle_id } }
}

#[component]
fn HandleList() -> Element {
    rsx! { pages::handles::HandleList {} }
}

#[component]
fn HandleShow(handle_id: String) -> Element {
    rsx! { pages::handles::HandleShow { handle_id } }
}

#[component]
fn PushRouteList() -> Element {
    rsx! { pages::push_routes::PushRoutes {} }
}

#[component]
fn RealmDeliveryBinding(realm_id: String) -> Element {
    rsx! { pages::delivery_binding::DeliveryBindingPolicy { realm_id } }
}

#[component]
fn RealmLinks(realm_id: String) -> Element {
    rsx! { pages::realm_links::RealmLinks { realm_id } }
}

#[component]
fn RealmMediaService(realm_id: String) -> Element {
    rsx! { pages::realm_media_service::RealmMediaService { realm_id } }
}

#[component]
fn RealmIdentityAudit(realm_id: String) -> Element {
    rsx! { pages::realm_identity_audit::RealmIdentityAudit { realm_id } }
}

#[component]
fn HandlesBySubject(subject: Option<String>) -> Element {
    rsx! { pages::handles_by_subject::HandlesBySubject { subject } }
}

#[component]
fn CoauthRecovery() -> Element {
    rsx! { pages::coauth::recovery::RecoveryPolicyList {} }
}

#[component]
fn AppletList() -> Element {
    rsx! { pages::applets::AppletList {} }
}

#[component]
fn AppletAdmin() -> Element {
    rsx! { pages::applets::admin::AppletAdminPage {} }
}

#[component]
fn AgentList() -> Element {
    rsx! { pages::agents::list::AgentList {} }
}

#[component]
fn AgentAdmin() -> Element {
    rsx! { pages::agents::admin::AgentAdminPage {} }
}

#[component]
fn DirectoryAdmin() -> Element {
    rsx! { pages::directory::admin::DirectoryAdminPage {} }
}

#[component]
fn RealmFederationStatus(realm_id: String) -> Element {
    rsx! { pages::realms::federation_status::FederationStatusPage { realm_id } }
}

#[component]
fn RealmPolicyEditor(realm_id: String) -> Element {
    rsx! { pages::realms::policy_editor::PolicyEditorPage { realm_id } }
}

#[component]
fn AgentShow(agent_id: String) -> Element {
    rsx! { pages::agents::show::AgentShow { agent_id } }
}

#[component]
fn PersonalAgentList() -> Element {
    rsx! { pages::agents::personal::PersonalAgentList {} }
}

#[component]
fn PersonalAgentShow(agent_id: String) -> Element {
    rsx! { pages::agents::personal::PersonalAgentShow { agent_id } }
}

#[component]
fn KeyBackupList() -> Element {
    rsx! { pages::key_backup::KeyBackupList {} }
}

#[component]
fn AuditLog() -> Element {
    rsx! { pages::audit::AuditLog {} }
}

#[component]
fn InviteTokenList() -> Element {
    rsx! { pages::invite_tokens::InviteTokenList {} }
}

#[component]
fn ThirdPartyInvites() -> Element {
    rsx! { pages::invites_3pid::ThirdPartyInvitesPage {} }
}

#[component]
fn PolicyList() -> Element {
    rsx! { pages::policy::PolicyList {} }
}

#[component]
fn ServerStatus() -> Element {
    rsx! { pages::server_status::ServerStatus {} }
}

#[component]
fn HardeningDashboard() -> Element {
    rsx! { pages::hardening::HardeningDashboard {} }
}

#[component]
fn StaridResolver() -> Element {
    rsx! { pages::starid_resolver::StaridResolverPage {} }
}

#[component]
fn CoauthAuditLog() -> Element {
    rsx! { pages::coauth::audit_log::AuditLogPage {} }
}

#[component]
fn CoauthAccountList() -> Element {
    rsx! { pages::coauth::accounts::AccountsPage {} }
}

#[component]
fn CoauthAccountShow(account_id: String) -> Element {
    rsx! { pages::coauth::account_detail::AccountDetailPage { account_id } }
}

#[component]
fn CoauthAccountDevices(account_id: String) -> Element {
    rsx! { pages::coauth::devices::AccountDevicesPage { account_id } }
}

#[component]
fn CoauthCapabilities() -> Element {
    rsx! { pages::coauth::capabilities::AuthzCapabilitiesPage {} }
}

#[component]
fn CoauthOAuth2Sessions() -> Element {
    rsx! { pages::coauth::oauth2_sessions::OAuth2SessionsPage {} }
}

#[component]
fn CoauthPersonalSessions() -> Element {
    rsx! { pages::coauth::personal_sessions::PersonalSessionsPage {} }
}

#[component]
fn CoauthUpstreamProviders() -> Element {
    rsx! { pages::coauth::upstream_providers::UpstreamProvidersPage {} }
}

#[component]
fn CoauthUpstreamLinks() -> Element {
    rsx! { pages::coauth::upstream_links::UpstreamLinksPage {} }
}

#[component]
fn CoauthRegistrationTokens() -> Element {
    rsx! { pages::coauth::registration_tokens::RegistrationTokensPage {} }
}

#[component]
fn CoauthNotificationChannels() -> Element {
    rsx! { pages::coauth::notification_channels::NotificationChannelsPage {} }
}

#[component]
fn CoauthNotificationTemplates() -> Element {
    rsx! { pages::coauth::notification_templates::NotificationTemplatesPage {} }
}

#[component]
fn CoauthConnectorHealth() -> Element {
    rsx! { pages::coauth::connector_health::ConnectorHealthPage {} }
}

#[component]
fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-center",
                h1 { class: "text-4xl font-bold mb-4", {t("not_found.title")} }
                p { class: "text-muted-foreground mb-4",
                    {format!("{}: /{}", t("not_found.message"), route.join("/"))}
                }
                Link { to: Route::Dashboard {}, class: "text-primary hover:underline",
                    {t("not_found.go_dashboard")}
                }
            }
        }
    }
}
