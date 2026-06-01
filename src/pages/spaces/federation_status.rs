//! Federation status admin page
//!
//! Per-Space federation peers + last-anchor-pulled-at + outbound queue
//! depth. Reads from soland's `GET /api/admin/v1/federation/status`
//! (404-tolerant — surfaces "endpoint not yet wired" toast).

use coauth_admin_types::federation_admin::FederationPeerHealth;
use dioxus::prelude::*;

use crate::api::federation_status_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

#[component]
pub fn FederationStatusPage(space_id: String) -> Element {
    let id_for_resource = space_id.clone();
    let id_filter = if space_id == "_" {
        None
    } else {
        Some(id_for_resource.clone())
    };

    let mut data = use_resource(move || {
        let id = id_filter.clone();
        async move { federation_status_admin::get_status(id.as_deref()).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("federation_status.title"),
                description: t("federation_status.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            match &*data.read() {
                Some(Ok(env)) => {
                    let rows = env.data.clone();
                    let generated = env.generated_at.clone().unwrap_or_default();
                    rsx! {
                        if rows.is_empty() {
                            EmptyState {
                                icon: "globe".to_string(),
                                title: t("federation_status.empty_title"),
                                description: t("federation_status.empty_subtitle"),
                            }
                        } else {
                            if !generated.is_empty() {
                                p { class: "text-xs text-muted-foreground",
                                    {format!("{}: {generated}", t("federation_status.generated_at"))}
                                }
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("federation_status.space_id")} }
                                            TableHead { {t("federation_status.peer")} }
                                            TableHead { {t("federation_status.health")} }
                                            TableHead { {t("federation_status.last_anchor_pulled_at")} }
                                            TableHead { {t("federation_status.last_pushed_at")} }
                                            TableHead { {t("federation_status.outbound_queue_depth")} }
                                        }
                                    }
                                    TableBody {
                                        for r in rows.iter() {
                                            {
                                                let typed = r.health_typed();
                                                let label = typed.label().to_string();
                                                let variant = peer_health_variant(&typed);
                                                let peer_label = r.peer_label.clone().unwrap_or_default();
                                                let pulled = r.last_anchor_pulled_at.clone().unwrap_or_else(|| "-".to_string());
                                                let pushed = r.last_pushed_at.clone().unwrap_or_else(|| "-".to_string());
                                                let depth = r.outbound_queue_depth;
                                                let space = r.space_id.clone();
                                                let peer = r.peer_did.clone();
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{space}" }
                                                        TableCell {
                                                            div { class: "flex flex-col",
                                                                if !peer_label.is_empty() {
                                                                    span { class: "font-medium", "{peer_label}" }
                                                                }
                                                                span { class: "font-mono text-xs text-muted-foreground", "{peer}" }
                                                            }
                                                        }
                                                        TableCell { Badge { variant, "{label}" } }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{pulled}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{pushed}" }
                                                        TableCell { "{depth}" }
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
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

/// Pick a Badge variant for a peer health bucket. Pure helper.
pub(crate) fn peer_health_variant(h: &FederationPeerHealth) -> BadgeVariant {
    match h {
        FederationPeerHealth::Healthy => BadgeVariant::Success,
        FederationPeerHealth::Degraded => BadgeVariant::Secondary,
        FederationPeerHealth::Unreachable => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_buckets_match_severity() {
        assert!(matches!(
            peer_health_variant(&FederationPeerHealth::Healthy),
            BadgeVariant::Success
        ));
        assert!(matches!(
            peer_health_variant(&FederationPeerHealth::Degraded),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            peer_health_variant(&FederationPeerHealth::Unreachable),
            BadgeVariant::Destructive
        ));
    }
}
