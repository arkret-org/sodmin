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
        #[route("/actors/:actor_id")]
        ActorShow { actor_id: String },

        #[route("/realms")]
        RealmList {},
        #[route("/realms/create")]
        RealmCreate {},
        #[route("/realms/:realm_id")]
        RealmShow { realm_id: String },
        #[route("/realms/:realm_id/notary")]
        RealmNotary { realm_id: String },
        #[route("/realms/:realm_id/seal-dag")]
        RealmSealDag { realm_id: String },
        #[route("/realms/:realm_id/covered-seals")]
        RealmCoveredSeals { realm_id: String },
        #[route("/realms/:realm_id/signing-keys")]
        RealmSigningKeys { realm_id: String },
        #[route("/realms/:realm_id/multisig")]
        RealmMultiSig { realm_id: String },
        #[route("/realms/:realm_id/federation-status")]
        RealmFederationStatus { realm_id: String },

        #[route("/spaces")]
        SpaceList {},
        #[route("/spaces/:space_id")]
        SpaceShow { space_id: String },

        #[route("/seal/bottom")]
        SealBottom {},

        #[route("/media")]
        MediaList {},

        #[route("/federation")]
        FederationList {},
        #[route("/federation/:domain")]
        FederationShow { domain: String },

        #[route("/devices")]
        DeviceList {},

        #[route("/capabilities")]
        CapabilityList {},

        #[route("/handles")]
        HandleList {},
        #[route("/handles/:handle_id")]
        HandleShow { handle_id: String },

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
        #[route("/realms/:realm_id/identity-audit")]
        RealmIdentityAudit { realm_id: String },

        // R3.2 (UI-SOD-4) — Subject → Handles directory page. Operator
        // enters a holder/principal DID; the page calls
        // `ck.find.directory.query.list_handles_for_subject` and lists the visible
        // signed handle claims + the §3.2.1 primary handle.
        #[route("/handles/by-subject?:subject")]
        HandlesBySubject { subject: Option<String> },

        // B-C key-backup admin surface (P3-B).
        #[route("/key-backup")]
        KeyBackupList {},

        #[route("/audit")]
        AuditLog {},

        #[route("/invite-tokens")]
        InviteTokenList {},

        #[route("/policy")]
        PolicyList {},

        #[route("/server-status")]
        ServerStatus {},

        #[route("/hardening")]
        HardeningDashboard {},

        // Round R2/R3 T07 — deactivation 7-domain fanout review.
        #[route("/deactivations/review")]
        DeactivationReview {},
        // Round R2/R3 T07 — realm destroy confirmation + post-seal
        // fanout + erasure receipt panel.
        #[route("/realms/:realm_id/destroy")]
        RealmDestroy { realm_id: String },
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
fn RealmNotary(realm_id: String) -> Element {
    rsx! { pages::realms::notary::NotaryPage { realm_id } }
}

#[component]
fn RealmSealDag(realm_id: String) -> Element {
    rsx! { pages::realms::seal_dag::SealDagPage { realm_id } }
}

#[component]
fn SealBottom() -> Element {
    rsx! { pages::seal_bottom::BottomDiagnosticsPage {} }
}

#[component]
fn RealmCoveredSeals(realm_id: String) -> Element {
    rsx! { pages::realms::covered_seals::CoveredSealsPage { realm_id } }
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
fn MediaList() -> Element {
    rsx! { pages::media::MediaList {} }
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
fn HandleList() -> Element {
    rsx! { pages::handles::HandleList {} }
}

#[component]
fn HandleShow(handle_id: String) -> Element {
    rsx! { pages::handles::HandleShow { handle_id } }
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
fn RealmFederationStatus(realm_id: String) -> Element {
    rsx! { pages::realms::federation_status::FederationStatusPage { realm_id } }
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
