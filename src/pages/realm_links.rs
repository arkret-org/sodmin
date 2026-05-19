//! R5.2 — Realm link-graph admin page (simple list view).
//!
//! Renders the outbound (`this Realm → others`) and inbound
//! (`others → this Realm`) `cx.realm.link` rows for a single Realm.
//! Each row is a card with a chip for the `link_kind`
//! (`governed_by`, `discoverable_from`, `mirror_of`, …) and the
//! target Realm identifier.
//!
//! This is intentionally the "list + chip" version of the feature.
//! TODO(realm-rework): replace the lists with an SVG / canvas
//! link-graph visualisation once the graph layout component lands.

use dioxus::prelude::*;

use crate::api::realm_links::{self, LinkDirection};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::types::RealmLinkRow;
use crate::utils::i18n::t;

/// Map a free-form `link_kind` to the badge variant used to colour the
/// chip. Unknown kinds fall back to the neutral `Secondary` variant so
/// the surface keeps rendering when soland introduces a new link
/// shape.
fn link_kind_variant(link_kind: &str) -> BadgeVariant {
    match link_kind {
        "governed_by" => BadgeVariant::Default,
        "discoverable_from" => BadgeVariant::Secondary,
        "mirror_of" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

/// Look up the i18n label for a `link_kind`, falling back to the raw
/// wire string so unknown kinds remain debuggable.
fn link_kind_label(link_kind: &str) -> String {
    let key = format!("realm_links.kind.{}", link_kind);
    let translated = t(&key);
    if translated == key {
        link_kind.to_string()
    } else {
        translated
    }
}

#[component]
pub fn RealmLinks(realm_id: String) -> Element {
    let id_outbound = realm_id.clone();
    let id_inbound = realm_id.clone();

    let mut outbound = use_resource(move || {
        let id = id_outbound.clone();
        async move { realm_links::list_realm_links(&id, LinkDirection::Outbound).await }
    });
    let mut inbound = use_resource(move || {
        let id = id_inbound.clone();
        async move { realm_links::list_realm_links(&id, LinkDirection::Inbound).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_links.title"),
                description: format!("{}: {}", t("realm_links.realm"), realm_id),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        outbound.restart();
                        inbound.restart();
                    },
                    {t("common.refresh")}
                }
            }

            // Outbound — this Realm points at others.
            Card {
                CardHeader {
                    CardTitle { {t("realm_links.outbound_title")} }
                    CardDescription { {t("realm_links.outbound_subtitle")} }
                }
                CardContent {
                    {render_link_list(&*outbound.read(), move |_| outbound.restart(), true)}
                }
            }

            // Inbound — others point at this Realm.
            Card {
                CardHeader {
                    CardTitle { {t("realm_links.inbound_title")} }
                    CardDescription { {t("realm_links.inbound_subtitle")} }
                }
                CardContent {
                    {render_link_list(&*inbound.read(), move |_| inbound.restart(), false)}
                }
            }

            // Placeholder for the future graph visualisation. We keep
            // it here so the page layout stays consistent with the
            // eventual richer view.
            Card {
                CardHeader {
                    CardTitle { {t("realm_links.graph_title")} }
                    CardDescription { {t("realm_links.graph_subtitle")} }
                }
                CardContent {
                    p { class: "text-sm text-muted-foreground py-6 text-center",
                        {t("realm_links.graph_placeholder")}
                    }
                }
            }
        }
    }
}

fn render_link_list<F>(
    data: &Option<Result<crate::types::ListResponse<RealmLinkRow>, crate::utils::error::HttpError>>,
    retry: F,
    outbound: bool,
) -> Element
where
    F: FnMut(MouseEvent) + 'static + Copy,
{
    match data {
        Some(Ok(resp)) => {
            if resp.data.is_empty() {
                let empty_key = if outbound {
                    "realm_links.outbound_empty"
                } else {
                    "realm_links.inbound_empty"
                };
                rsx! {
                    p { class: "text-sm text-muted-foreground py-4 text-center",
                        {t(empty_key)}
                    }
                }
            } else {
                rsx! {
                    div { class: "space-y-2",
                        for row in resp.data.iter() {
                            {render_link_row(row, outbound)}
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            ErrorBanner { message: e.message.clone(), on_retry: retry }
        },
        None => rsx! { PageSkeleton {} },
    }
}

fn render_link_row(row: &RealmLinkRow, outbound: bool) -> Element {
    let other_realm = if outbound {
        row.target_realm_id.clone()
    } else {
        row.source_realm_id.clone()
    };
    let label = link_kind_label(&row.link_kind);
    let variant = link_kind_variant(&row.link_kind);
    let updated = row.updated_at.clone().unwrap_or_else(|| "-".to_string());
    let display_name = row.target_display_name.clone();

    rsx! {
        div {
            class: "flex flex-wrap items-center gap-2 rounded-md border border-border bg-card px-3 py-2 text-sm",
            Badge {
                variant: variant,
                class: "text-xs".to_string(),
                "{label}"
            }
            span { class: "font-mono text-xs break-all", "{other_realm}" }
            if let Some(name) = display_name.as_ref() {
                span { class: "text-xs text-muted-foreground", "({name})" }
            }
            span { class: "ml-auto text-xs text-muted-foreground", "{updated}" }
        }
    }
}
