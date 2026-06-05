use dioxus::prelude::*;

use crate::components::ui::icons::Icon;
use crate::router::Route;
use crate::utils::i18n::t;
use crate::utils::net::session::{self, bridge, scope};

struct NavItem {
    title: String,
    route: Route,
    icon: &'static str,
    /// Required scope this nav item needs. `None` means "always allowed
    /// when the parent group is shown".
    required_scope: Option<&'static str>,
}

impl NavItem {
    fn new(title: String, route: Route, icon: &'static str) -> Self {
        Self {
            title,
            route,
            icon,
            required_scope: None,
        }
    }

    fn scoped(mut self, scope: &'static str) -> Self {
        self.required_scope = Some(scope);
        self
    }
}

struct NavSection {
    label: String,
    items: Vec<NavItem>,
    /// Required active bridge contract for the whole group. `None` means
    /// "always shown".
    required_bridge: Option<&'static str>,
    /// Required admin scope for the whole group. `None` means "show iff
    /// at least one inner item is allowed by `has_scope`".
    required_scope: Option<&'static str>,
}

impl NavSection {
    fn new(label: String, items: Vec<NavItem>) -> Self {
        Self {
            label,
            items,
            required_bridge: None,
            required_scope: None,
        }
    }

    fn bridge(mut self, b: &'static str) -> Self {
        self.required_bridge = Some(b);
        self
    }

    fn scope(mut self, s: &'static str) -> Self {
        self.required_scope = Some(s);
        self
    }

    /// Item filter — keep only entries whose required_scope is held by
    /// the current session.
    fn filtered_items(&self) -> Vec<&NavItem> {
        self.items
            .iter()
            .filter(|item| match item.required_scope {
                Some(s) => session::has_scope(s),
                None => true,
            })
            .collect()
    }

    /// Hide a whole section if its bridge is not active or its scope is
    /// not held — also hide if every item has been filtered out.
    fn is_visible(&self) -> bool {
        if let Some(b) = self.required_bridge
            && !session::has_bridge(b)
        {
            return false;
        }
        if let Some(s) = self.required_scope
            && !session::has_scope(s)
        {
            return false;
        }
        !self.filtered_items().is_empty()
    }
}

/// Build the dynamic sidebar nav. Reads `active_bridges` + `admin_scope`
/// from the cached session info and only emits groups/items the operator
/// is actually allowed to see. Unknown extras in either set are ignored.
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

    sections.push(
        NavSection::new(
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
                NavItem::new(t("nav.push_routes"), Route::PushRouteList {}, "smartphone"),
                NavItem::new(t("nav.capabilities"), Route::CapabilityList {}, "shield"),
                NavItem::new(
                    t("nav.coauth_capabilities"),
                    Route::CoauthCapabilities {},
                    "shield",
                ),
                NavItem::new(t("nav.invite_tokens"), Route::InviteTokenList {}, "key"),
                // Round 4 — 3PID third-party invite state-machine view.
                NavItem::new(t("nav.invites_3pid"), Route::ThirdPartyInvites {}, "mail"),
            ],
        )
        .bridge(bridge::SOLAND)
        .scope(scope::IDENTITY),
    );

    // CKP-0007 Circles — encrypted sub-boundary admin (P3A.3).
    // Pinned right under Identity so operators see Circles next to
    // the Realm membership surfaces they extend.
    sections.push(
        NavSection::new(
            t("nav.section_circles"),
            vec![NavItem::new(
                t("nav.circles"),
                Route::CircleList {},
                "users",
            )],
        )
        .bridge(bridge::SOLAND)
        .scope(scope::IDENTITY),
    );

    sections.push(
        NavSection::new(
            t("nav.section_moderation"),
            vec![
                NavItem::new(t("nav.realms"), Route::RealmList {}, "shield"),
                NavItem::new(t("nav.spaces"), Route::SpaceList {}, "message-square"),
                NavItem::new(t("nav.reports"), Route::ReportList {}, "flag"),
                NavItem::new(
                    t("nav.moderation_reports"),
                    Route::ModerationReports {},
                    "flag",
                ),
                // Round R2/R3 T06 — moderation appeal admin.
                NavItem::new(
                    "Moderation appeals".to_string(),
                    Route::ModerationAppeals {},
                    "flag",
                ),
                NavItem::new(t("nav.audit"), Route::AuditLog {}, "scroll-text"),
                // Round R2/R3 T10 — attestation evidence admin.
                NavItem::new(
                    "Audit attestation".to_string(),
                    Route::AuditAttestation {},
                    "shield",
                ),
            ],
        )
        .bridge(bridge::SOLAND)
        .scope(scope::MODERATION),
    );

    sections.push(
        NavSection::new(
            t("nav.section_infrastructure"),
            vec![
                NavItem::new(t("nav.federation"), Route::FederationList {}, "globe"),
                NavItem::new(t("nav.media"), Route::MediaList {}, "image"),
                NavItem::new(t("nav.applets"), Route::AppletList {}, "plug"),
                NavItem::new(t("nav.applets_admin"), Route::AppletAdmin {}, "plug"),
                NavItem::new(t("nav.agents"), Route::AgentList {}, "bot"),
                NavItem::new(t("nav.agents_admin"), Route::AgentAdmin {}, "bot"),
                // CKP-0008 personal-agent admin (P3-A).
                NavItem::new(
                    "Personal agents".to_string(),
                    Route::PersonalAgentList {},
                    "bot",
                ),
                // B-C key-backup recovery admin (P3-B).
                NavItem::new("Key backup".to_string(), Route::KeyBackupList {}, "key"),
                NavItem::new(t("nav.directory_admin"), Route::DirectoryAdmin {}, "globe"),
                NavItem::new(
                    t("nav.federation_status"),
                    Route::RealmFederationStatus {
                        realm_id: "_".to_string(),
                    },
                    "globe",
                ),
                NavItem::new(
                    t("nav.policy_editor"),
                    Route::RealmPolicyEditor {
                        realm_id: "_".to_string(),
                    },
                    "file-text",
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
                // R3 (UI-3) — Realm media_service.foci[] editor.
                NavItem::new(
                    t("media_service.title"),
                    Route::RealmMediaService {
                        realm_id: "_".to_string(),
                    },
                    "video",
                ),
            ],
        )
        .bridge(bridge::SOLAND)
        .scope(scope::INFRASTRUCTURE),
    );

    sections.push(
        NavSection::new(
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
                // Round R2/R3 T08 — deployment-wide trust_domain edit.
                NavItem::new(
                    "Trust domain".to_string(),
                    Route::TrustDomainConfig {},
                    "shield",
                ),
                // Round R2/R3 T09 — relaxed ephemeral window slider.
                NavItem::new(
                    "Relaxed window".to_string(),
                    Route::RelaxedWindow {},
                    "activity",
                ),
                // Round R2/R3 T07 — deactivation fanout review.
                NavItem::new(
                    "Deactivation review".to_string(),
                    Route::DeactivationReview {},
                    "alert-triangle",
                ),
            ],
        )
        .bridge(bridge::SOLAND)
        .scope(scope::SERVER_OPS),
    );

    // Stream H' (Move/Anchor/Lattice admin) — gated on the soland bridge
    // (without the principal server there is no Move/Anchor surface) and
    // on the anchor admin scope. Realm deep links keep a placeholder id
    // because admins typically arrive from the Realm detail page.
    sections.push(
        NavSection::new(
            t("nav.section_anchor"),
            vec![
                NavItem::new(
                    t("nav.anchor_bottom"),
                    Route::AnchorBottom {},
                    "alert-triangle",
                ),
                NavItem::new(
                    t("nav.anchor_anchorer"),
                    Route::RealmAnchorer {
                        realm_id: "_".to_string(),
                    },
                    "shield",
                ),
                NavItem::new(
                    t("nav.anchor_dag"),
                    Route::RealmAnchorDag {
                        realm_id: "_".to_string(),
                    },
                    "git-branch",
                ),
                NavItem::new(
                    t("nav.consent"),
                    Route::RealmConsent {
                        realm_id: "_".to_string(),
                    },
                    "shield",
                ),
                NavItem::new(
                    t("nav.covered_frontier"),
                    Route::RealmCoveredFrontier {
                        realm_id: "_".to_string(),
                    },
                    "lock",
                ),
                NavItem::new(t("nav.components"), Route::ComponentsRegistry {}, "plug"),
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
        )
        .bridge(bridge::SOLAND)
        .scope(scope::ANCHOR),
    );

    sections.push(
        NavSection::new(
            t("nav.section_coauth"),
            vec![
                NavItem::new(
                    t("nav.coauth_accounts"),
                    Route::CoauthAccountList {},
                    "user-round",
                )
                .scoped(scope::COAUTH),
                NavItem::new(t("nav.audit_log"), Route::CoauthAuditLog {}, "scroll-text")
                    .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.oauth2_sessions"),
                    Route::CoauthOAuth2Sessions {},
                    "key",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.personal_tokens"),
                    Route::CoauthPersonalSessions {},
                    "fingerprint",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.registration_tokens"),
                    Route::CoauthRegistrationTokens {},
                    "ticket",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.upstream_providers"),
                    Route::CoauthUpstreamProviders {},
                    "link",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.upstream_links"),
                    Route::CoauthUpstreamLinks {},
                    "link",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.notification_channels"),
                    Route::CoauthNotificationChannels {},
                    "mail",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.notification_templates"),
                    Route::CoauthNotificationTemplates {},
                    "scroll-text",
                )
                .scoped(scope::COAUTH),
                NavItem::new(
                    t("nav.connector_health"),
                    Route::CoauthConnectorHealth {},
                    "heart-pulse",
                )
                .scoped(scope::COAUTH),
                // R3 (UI-4) — recovery policies + receipts stub.
                NavItem::new(t("recovery.title"), Route::CoauthRecovery {}, "shield")
                    .scoped(scope::COAUTH),
            ],
        )
        .bridge(bridge::COAUTH),
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
                        for item in section.filtered_items().into_iter() {
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
        Route::ActorList {} => matches!(
            current,
            Route::ActorList {} | Route::ActorShow { .. } | Route::ActorCreate {}
        ),
        Route::HandleList {} => matches!(current, Route::HandleList {} | Route::HandleShow { .. }),
        Route::RealmList {} => matches!(
            current,
            Route::RealmList {} | Route::RealmShow { .. } | Route::RealmCreate { .. }
        ),
        Route::SpaceList {} => matches!(current, Route::SpaceList {} | Route::SpaceShow { .. }),
        Route::ReportList {} => matches!(current, Route::ReportList {} | Route::ReportShow { .. }),
        Route::ModerationReports {} => matches!(current, Route::ModerationReports {}),
        Route::FederationList {} => matches!(
            current,
            Route::FederationList {} | Route::FederationShow { .. }
        ),
        Route::AgentList {} => matches!(current, Route::AgentList {} | Route::AgentShow { .. }),
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

    /// Apply the same filtering as `NavSection::filtered_items` but using
    /// an explicit (storage-free) scope set so the unit tests don't need
    /// browser `LocalStorage`. Mirrors the runtime semantics exactly.
    fn filter_with_scopes<'a>(section: &'a NavSection, scopes: &[&str]) -> Vec<&'a NavItem> {
        section
            .items
            .iter()
            .filter(|item| match item.required_scope {
                Some(s) => scopes.contains(&scope::WILDCARD) || scopes.contains(&s),
                None => true,
            })
            .collect()
    }

    fn section_visible_with(section: &NavSection, bridges: &[&str], scopes: &[&str]) -> bool {
        if let Some(b) = section.required_bridge
            && !bridges.contains(&b)
        {
            return false;
        }
        if let Some(s) = section.required_scope
            && !(scopes.contains(&scope::WILDCARD) || scopes.contains(&s))
        {
            return false;
        }
        !filter_with_scopes(section, scopes).is_empty()
    }

    #[test]
    fn nav_section_hidden_when_required_bridge_inactive() {
        let section = NavSection::new(
            "x".to_string(),
            vec![NavItem::new("y".to_string(), Route::Dashboard {}, "x")],
        )
        .bridge(bridge::SOLAND);
        assert!(!section_visible_with(&section, &[], &[scope::WILDCARD]));
        assert!(section_visible_with(
            &section,
            &[bridge::SOLAND],
            &[scope::WILDCARD]
        ));
    }

    #[test]
    fn nav_section_filters_items_by_scope() {
        let section = NavSection::new(
            "x".to_string(),
            vec![
                NavItem::new("a".to_string(), Route::Dashboard {}, "x"),
                NavItem::new("b".to_string(), Route::Dashboard {}, "x").scoped(scope::COAUTH),
            ],
        );
        // Wildcard sees both items.
        assert_eq!(filter_with_scopes(&section, &[scope::WILDCARD]).len(), 2);
        // No scope at all hides the coauth-scoped item.
        assert_eq!(filter_with_scopes(&section, &[]).len(), 1);
        // Explicit coauth scope sees both.
        assert_eq!(filter_with_scopes(&section, &[scope::COAUTH]).len(), 2);
    }

    #[test]
    fn nav_section_hidden_when_all_items_filtered_out() {
        let section = NavSection::new(
            "x".to_string(),
            vec![NavItem::new("y".to_string(), Route::Dashboard {}, "x").scoped(scope::COAUTH)],
        );
        assert!(!section_visible_with(&section, &[], &[]));
    }
}
