use dioxus::prelude::*;

use crate::components::ui::icons::Icon;
use crate::router::Route;
use crate::utils::i18n::t;
use crate::utils::net::session;

struct NavItem {
    title: String,
    route: Route,
    icon: &'static str,
}

impl NavItem {
    fn new(title: String, route: Route, icon: &'static str) -> Self {
        Self { title, route, icon }
    }
}

struct NavSection {
    label: String,
    items: Vec<NavItem>,
    /// True for sections that only make sense when a coauth upstream is
    /// configured (auto-derived from `session::has_coauth()`; server-side
    /// RBAC remains the authorization authority).
    requires_coauth: bool,
}

impl NavSection {
    fn new(label: String, items: Vec<NavItem>) -> Self {
        Self {
            label,
            items,
            requires_coauth: false,
        }
    }

    fn coauth(mut self) -> Self {
        self.requires_coauth = true;
        self
    }

    fn is_visible(&self) -> bool {
        !self.requires_coauth || session::has_coauth()
    }
}

/// Build the sidebar nav. The only dynamic gate is the coauth section,
/// which hides itself when no coauth upstream is configured.
fn build_nav_sections() -> Vec<NavSection> {
    // Dashboard is always visible — it has its own per-bridge readiness
    // surface.
    let mut sections: Vec<NavSection> = vec![NavSection::new(
        String::new(),
        vec![NavItem::new(
            t("nav.dashboard"),
            Route::Dashboard {},
            "layout-dashboard",
        )],
    )];

    sections.push(NavSection::new(
        t("nav.section_identity"),
        vec![
            NavItem::new(t("nav.actors"), Route::ActorList {}, "users"),
            NavItem::new(t("nav.devices"), Route::DeviceList {}, "smartphone"),
            NavItem::new(t("nav.handles"), Route::HandleList {}, "fingerprint"),
            NavItem::new(
                t("nav.handles_by_subject"),
                Route::HandlesBySubject { subject: None },
                "search",
            ),
            NavItem::new(t("nav.capabilities"), Route::CapabilityList {}, "shield"),
            NavItem::new(
                t("nav.coauth_capabilities"),
                Route::CoauthCapabilities {},
                "shield",
            ),
            NavItem::new(t("nav.invite_tokens"), Route::InviteTokenList {}, "key"),
        ],
    ));

    sections.push(NavSection::new(
        t("nav.section_moderation"),
        vec![
            NavItem::new(t("nav.realms"), Route::RealmList {}, "shield"),
            NavItem::new(t("nav.spaces"), Route::SpaceList {}, "message-square"),
            NavItem::new(t("nav.audit"), Route::AuditLog {}, "scroll-text"),
        ],
    ));

    sections.push(NavSection::new(
        t("nav.section_infrastructure"),
        vec![
            NavItem::new(t("nav.federation"), Route::FederationList {}, "globe"),
            NavItem::new(t("nav.media"), Route::MediaList {}, "image"),
            // B-C key-backup recovery admin (P3-B).
            NavItem::new("Key backup".to_string(), Route::KeyBackupList {}, "key"),
            NavItem::new(
                t("nav.federation_status"),
                Route::RealmFederationStatus {
                    realm_id: "_".to_string(),
                },
                "globe",
            ),
            NavItem::new(
                t("nav.delivery_binding"),
                // Realm-rework: link to the Realm-scoped editor.
                Route::RealmDeliveryBinding {
                    realm_id: "_".to_string(),
                },
                "shield",
            ),
            // R5.2 — Realm link-graph (outbound / inbound
            // `ck.realm.link` rows). Sits next to delivery binding
            // so the operator can pivot from a single Realm's
            // routing policy to its boundary topology.
            NavItem::new(
                t("nav.realm_links"),
                Route::RealmLinks {
                    realm_id: "_".to_string(),
                },
                "link",
            ),
            // R3 (UI-3) — Realm media_service.foci[] read-only view.
            NavItem::new(
                t("media_service.title"),
                Route::RealmMediaService {
                    realm_id: "_".to_string(),
                },
                "video",
            ),
            // SOD-ORG-01..03 — Realm verified organization relationship +
            // organization principal control / delegation audit.
            NavItem::new(
                t("realm_organization.title"),
                Route::RealmOrganization {
                    realm_id: "_".to_string(),
                },
                "building",
            ),
        ],
    ));

    sections.push(NavSection::new(
        t("nav.section_server_ops"),
        vec![
            NavItem::new(t("nav.policy"), Route::PolicyList {}, "file-text"),
            NavItem::new(t("nav.server_status"), Route::ServerStatus {}, "activity"),
            NavItem::new(
                "Hardening".to_string(),
                Route::HardeningDashboard {},
                "shield",
            ),
            NavItem::new(
                t("nav.starid_resolver"),
                Route::StaridResolver {},
                "fingerprint",
            ),
            // Round R2/R3 T07 — deactivation fanout review.
            NavItem::new(
                "Deactivation review".to_string(),
                Route::DeactivationReview {},
                "alert-triangle",
            ),
        ],
    ));

    // Stream H' (Control-plane Seal admin). Realm deep links
    // keep a placeholder id because admins typically arrive from the
    // Realm detail page.
    sections.push(NavSection::new(
        t("nav.section_seal"),
        vec![
            NavItem::new(t("nav.seal_bottom"), Route::SealBottom {}, "alert-triangle"),
            NavItem::new(
                t("nav.seal_notary"),
                Route::RealmNotary {
                    realm_id: "_".to_string(),
                },
                "shield",
            ),
            NavItem::new(
                t("nav.seal_dag"),
                Route::RealmSealDag {
                    realm_id: "_".to_string(),
                },
                "git-branch",
            ),
            NavItem::new(
                t("nav.covered_seals"),
                Route::RealmCoveredSeals {
                    realm_id: "_".to_string(),
                },
                "lock",
            ),
            NavItem::new(
                t("nav.signing_keys"),
                Route::RealmSigningKeys {
                    realm_id: "_".to_string(),
                },
                "key",
            ),
            NavItem::new(
                t("nav.multisig"),
                Route::RealmMultiSig {
                    realm_id: "_".to_string(),
                },
                "users",
            ),
        ],
    ));

    sections.push(
        NavSection::new(
            t("nav.section_coauth"),
            vec![
                NavItem::new(
                    t("nav.coauth_accounts"),
                    Route::CoauthAccountList {},
                    "user-round",
                ),
                NavItem::new(t("nav.audit_log"), Route::CoauthAuditLog {}, "scroll-text"),
                NavItem::new(
                    t("nav.oauth2_sessions"),
                    Route::CoauthOAuth2Sessions {},
                    "key",
                ),
                NavItem::new(
                    t("nav.personal_tokens"),
                    Route::CoauthPersonalSessions {},
                    "fingerprint",
                ),
                NavItem::new(
                    t("nav.registration_tokens"),
                    Route::CoauthRegistrationTokens {},
                    "ticket",
                ),
                NavItem::new(
                    t("nav.upstream_providers"),
                    Route::CoauthUpstreamProviders {},
                    "link",
                ),
                NavItem::new(
                    t("nav.upstream_links"),
                    Route::CoauthUpstreamLinks {},
                    "link",
                ),
                NavItem::new(
                    t("nav.notification_channels"),
                    Route::CoauthNotificationChannels {},
                    "mail",
                ),
                NavItem::new(
                    t("nav.notification_templates"),
                    Route::CoauthNotificationTemplates {},
                    "scroll-text",
                ),
                NavItem::new(
                    t("nav.connector_health"),
                    Route::CoauthConnectorHealth {},
                    "heart-pulse",
                ),
            ],
        )
        .coauth(),
    );

    sections.into_iter().filter(|s| s.is_visible()).collect()
}

#[component]
pub fn AppSidebar(collapsed: Signal<bool>, mobile_open: Signal<bool>) -> Element {
    let mut mobile_open = mobile_open;
    let nav = use_navigator();
    let current_path = use_route::<Route>();

    let sections = build_nav_sections();

    let is_mobile_open = *mobile_open.read();
    let is_collapsed = *collapsed.read() && !is_mobile_open;
    let width_class = if is_collapsed { "w-16" } else { "w-64" };
    let mobile_state_class = if is_mobile_open { "sidebar-open" } else { "" };
    let nav_item_layout_class = if is_collapsed {
        "justify-center px-0"
    } else {
        "px-3"
    };

    rsx! {
        aside {
            class: "sidebar-shell sidebar-transition flex h-screen flex-col bg-sidebar border-r border-sidebar-border {width_class} {mobile_state_class}",

            div { class: "flex h-14 items-center gap-2 border-b border-sidebar-border px-4",
                if !is_collapsed {
                    div { class: "flex items-center gap-2",
                        Icon { name: "shield".to_string(), class: "h-6 w-6 text-sidebar-primary".to_string() }
                        span { class: "truncate font-semibold text-sidebar-foreground", "Cokret Admin" }
                    }
                } else {
                    div { class: "flex justify-center w-full",
                        Icon { name: "shield".to_string(), class: "h-6 w-6 text-sidebar-primary".to_string() }
                    }
                }
                button {
                    class: "sidebar-mobile-close inline-flex h-9 w-9 items-center justify-center rounded-lg text-sidebar-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground touch-target",
                    title: t("header.close"),
                    onclick: move |_| mobile_open.set(false),
                    Icon { name: "x".to_string(), class: "h-4 w-4".to_string() }
                }
            }

            nav { class: "sidebar-nav flex-1 overflow-y-auto py-2",
                for section in sections.iter() {
                    div { class: "sidebar-section px-2 mb-1",
                        if !is_collapsed && !section.label.is_empty() {
                            p { class: "sidebar-section-label text-xs font-semibold text-sidebar-foreground/50 px-3 pt-3 pb-1 uppercase tracking-wider",
                                {section.label.clone()}
                            }
                        }
                        for item in section.items.iter() {
                            {
                                let is_active = is_route_active(&current_path, &item.route);
                                let active_class = if is_active {
                                    "sidebar-nav-active"
                                } else {
                                    ""
                                };
                                let route = item.route.clone();
                                let title = item.title.clone();
                                rsx! {
                                    button {
                                        class: "sidebar-nav-button flex w-full items-center gap-3 py-2 text-sm font-medium transition-colors {nav_item_layout_class} {active_class}",
                                        onclick: move |_| {
                                            mobile_open.set(false);
                                            let _ = nav.push(route.clone());
                                        },
                                        Icon { name: item.icon.to_string(), class: "h-4 w-4 shrink-0".to_string() }
                                        if !is_collapsed {
                                            span { "{title}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_route_active(current: &Route, target: &Route) -> bool {
    if current == target {
        return true;
    }

    match target {
        Route::ActorList {} => matches!(current, Route::ActorList {} | Route::ActorShow { .. }),
        Route::HandleList {} => matches!(current, Route::HandleList {} | Route::HandleShow { .. }),
        Route::RealmList {} => matches!(current, Route::RealmList {} | Route::RealmShow { .. }),
        Route::SpaceList {} => matches!(current, Route::SpaceList {} | Route::SpaceShow { .. }),
        Route::FederationList {} => matches!(
            current,
            Route::FederationList {} | Route::FederationShow { .. }
        ),
        Route::CoauthAccountList {} => matches!(
            current,
            Route::CoauthAccountList {} | Route::CoauthAccountShow { .. }
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Storage-free mirror of `NavSection::is_visible` so the unit test
    /// doesn't need browser `LocalStorage`.
    fn section_visible_with(section: &NavSection, coauth_configured: bool) -> bool {
        !section.requires_coauth || coauth_configured
    }

    #[test]
    fn coauth_section_hidden_without_coauth_upstream() {
        let section = NavSection::new(
            "x".to_string(),
            vec![NavItem::new("y".to_string(), Route::Dashboard {}, "x")],
        )
        .coauth();
        assert!(!section_visible_with(&section, false));
        assert!(section_visible_with(&section, true));
    }

    #[test]
    fn plain_section_always_visible() {
        let section = NavSection::new(
            "x".to_string(),
            vec![NavItem::new("y".to_string(), Route::Dashboard {}, "x")],
        );
        assert!(section_visible_with(&section, false));
    }
}
