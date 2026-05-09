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

        #[route("/spaces")]
        SpaceList {},
        #[route("/spaces/create")]
        SpaceCreate {},
        #[route("/spaces/:space_id")]
        SpaceShow { space_id: String },
        #[route("/spaces/:space_id/anchorer")]
        SpaceAnchorer { space_id: String },
        #[route("/spaces/:space_id/anchor-dag")]
        SpaceAnchorDag { space_id: String },
        #[route("/spaces/:space_id/consent")]
        SpaceConsent { space_id: String },
        #[route("/spaces/:space_id/covered-frontier")]
        SpaceCoveredFrontier { space_id: String },
        #[route("/spaces/:space_id/signing-keys")]
        SpaceSigningKeys { space_id: String },
        #[route("/spaces/:space_id/multisig")]
        SpaceMultiSig { space_id: String },

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

        #[route("/federation")]
        FederationList {},
        #[route("/federation/:domain")]
        FederationShow { domain: String },

        #[route("/devices")]
        DeviceList {},

        #[route("/capabilities")]
        CapabilityList {},

        #[route("/applets")]
        AppletList {},

        #[route("/agents")]
        AgentList {},
        #[route("/agents/:agent_id")]
        AgentShow { agent_id: String },

        #[route("/audit")]
        AuditLog {},

        #[route("/invite-tokens")]
        InviteTokenList {},

        #[route("/policy")]
        PolicyList {},

        #[route("/server-status")]
        ServerStatus {},

        #[route("/coauth/audit-log")]
        CoauthAuditLog {},
        #[route("/coauth/accounts")]
        CoauthAccountList {},
        #[route("/coauth/accounts/:account_id")]
        CoauthAccountShow { account_id: String },
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
            Err(_) => cached.or(Some(true)),
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
fn SpaceList() -> Element {
    rsx! { pages::spaces::list::SpaceList {} }
}

#[component]
fn SpaceCreate() -> Element {
    rsx! { pages::spaces::create::SpaceCreate {} }
}

#[component]
fn SpaceShow(space_id: String) -> Element {
    rsx! { pages::spaces::show::SpaceShow { space_id } }
}

#[component]
fn SpaceAnchorer(space_id: String) -> Element {
    rsx! { pages::spaces::anchorer::AnchorerPage { space_id } }
}

#[component]
fn SpaceAnchorDag(space_id: String) -> Element {
    rsx! { pages::spaces::anchor_dag::AnchorDagPage { space_id } }
}

#[component]
fn AnchorBottom() -> Element {
    rsx! { pages::spaces::bottom::BottomDiagnosticsPage {} }
}

#[component]
fn SpaceConsent(space_id: String) -> Element {
    rsx! { pages::spaces::consent::ConsentPage { space_id } }
}

#[component]
fn SpaceCoveredFrontier(space_id: String) -> Element {
    rsx! { pages::spaces::covered_frontier::CoveredFrontierPage { space_id } }
}

#[component]
fn SpaceSigningKeys(space_id: String) -> Element {
    rsx! { pages::spaces::signing_keys::SigningKeysPage { space_id } }
}

#[component]
fn SpaceMultiSig(space_id: String) -> Element {
    rsx! { pages::spaces::multisig::MultiSigPage { space_id } }
}

#[component]
fn ComponentsRegistry() -> Element {
    rsx! { pages::spaces::components::ComponentsPage {} }
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
fn AppletList() -> Element {
    rsx! { pages::applets::AppletList {} }
}

#[component]
fn AgentList() -> Element {
    rsx! { pages::agents::list::AgentList {} }
}

#[component]
fn AgentShow(agent_id: String) -> Element {
    rsx! { pages::agents::show::AgentShow { agent_id } }
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
