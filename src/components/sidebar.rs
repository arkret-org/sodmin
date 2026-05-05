use dioxus::prelude::*;

use crate::components::ui::icons::Icon;
use crate::router::Route;
use crate::utils::i18n::t;

struct NavItem {
    title: String,
    route: Route,
    icon: &'static str,
}

struct NavSection {
    label: String,
    items: Vec<NavItem>,
}

#[component]
pub fn AppSidebar(collapsed: Signal<bool>, mobile_open: Signal<bool>) -> Element {
    let mut mobile_open = mobile_open;
    let nav = use_navigator();
    let current_path = use_route::<Route>();

    let has_coauth = crate::utils::session::has_coauth();

    let mut sections: Vec<NavSection> = Vec::new();

    sections.push(NavSection {
        label: String::new(),
        items: vec![NavItem {
            title: t("nav.dashboard"),
            route: Route::Dashboard {},
            icon: "layout-dashboard",
        }],
    });

    sections.push(NavSection {
        label: t("nav.section_identity"),
        items: vec![
            NavItem {
                title: t("nav.actors"),
                route: Route::ActorList {},
                icon: "users",
            },
            NavItem {
                title: t("nav.devices"),
                route: Route::DeviceList {},
                icon: "smartphone",
            },
            NavItem {
                title: t("nav.capabilities"),
                route: Route::CapabilityList {},
                icon: "shield",
            },
            NavItem {
                title: t("nav.invite_tokens"),
                route: Route::InviteTokenList {},
                icon: "key",
            },
        ],
    });

    sections.push(NavSection {
        label: t("nav.section_moderation"),
        items: vec![
            NavItem {
                title: t("nav.spaces"),
                route: Route::SpaceList {},
                icon: "message-square",
            },
            NavItem {
                title: t("nav.reports"),
                route: Route::ReportList {},
                icon: "flag",
            },
            NavItem {
                title: t("nav.audit"),
                route: Route::AuditLog {},
                icon: "scroll-text",
            },
        ],
    });

    sections.push(NavSection {
        label: t("nav.section_infrastructure"),
        items: vec![
            NavItem {
                title: t("nav.federation"),
                route: Route::FederationList {},
                icon: "globe",
            },
            NavItem {
                title: t("nav.media"),
                route: Route::MediaList {},
                icon: "image",
            },
            NavItem {
                title: t("nav.applets"),
                route: Route::AppletList {},
                icon: "plug",
            },
            NavItem {
                title: t("nav.agents"),
                route: Route::AgentList {},
                icon: "bot",
            },
        ],
    });

    sections.push(NavSection {
        label: t("nav.section_server_ops"),
        items: vec![
            NavItem {
                title: t("nav.policy"),
                route: Route::PolicyList {},
                icon: "file-text",
            },
            NavItem {
                title: t("nav.server_status"),
                route: Route::ServerStatus {},
                icon: "activity",
            },
        ],
    });

    if has_coauth {
        sections.push(NavSection {
            label: t("nav.section_coauth"),
            items: vec![
                NavItem {
                    title: t("nav.coauth_accounts"),
                    route: Route::CoauthAccountList {},
                    icon: "user-round",
                },
                NavItem {
                    title: t("nav.audit_log"),
                    route: Route::CoauthAuditLog {},
                    icon: "scroll-text",
                },
                NavItem {
                    title: t("nav.oauth2_sessions"),
                    route: Route::CoauthOAuth2Sessions {},
                    icon: "key",
                },
                NavItem {
                    title: t("nav.personal_tokens"),
                    route: Route::CoauthPersonalSessions {},
                    icon: "fingerprint",
                },
                NavItem {
                    title: t("nav.registration_tokens"),
                    route: Route::CoauthRegistrationTokens {},
                    icon: "ticket",
                },
                NavItem {
                    title: t("nav.upstream_providers"),
                    route: Route::CoauthUpstreamProviders {},
                    icon: "link",
                },
                NavItem {
                    title: t("nav.upstream_links"),
                    route: Route::CoauthUpstreamLinks {},
                    icon: "link",
                },
                NavItem {
                    title: t("nav.notification_channels"),
                    route: Route::CoauthNotificationChannels {},
                    icon: "mail",
                },
                NavItem {
                    title: t("nav.notification_templates"),
                    route: Route::CoauthNotificationTemplates {},
                    icon: "scroll-text",
                },
                NavItem {
                    title: t("nav.connector_health"),
                    route: Route::CoauthConnectorHealth {},
                    icon: "heart-pulse",
                },
            ],
        });
    }

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
                        span { class: "truncate font-semibold text-sidebar-foreground", "Contrix Admin" }
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
    let current_str = format!("{:?}", current);
    let target_str = format!("{:?}", target);

    if current_str == target_str {
        return true;
    }

    match target {
        Route::ActorList {} => matches!(
            current,
            Route::ActorList {} | Route::ActorShow { .. } | Route::ActorCreate {}
        ),
        Route::SpaceList {} => matches!(current, Route::SpaceList {} | Route::SpaceShow { .. }),
        Route::ReportList {} => matches!(current, Route::ReportList {} | Route::ReportShow { .. }),
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
