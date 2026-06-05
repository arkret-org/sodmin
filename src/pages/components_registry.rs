//! Component criticality / version drift admin page (Stream H', H'6).
//!
//! Server-wide component registry view. Reads from
//! `GET /_soland/admin/components` and renders one row per
//! `ck.component.*` type with the cell_family it projects into, the
//! criticality classification, the spec version pinned by the bundle, the
//! impl version actually loaded, and an Active/Stub/Disabled status.
//! When `spec_version` and `impl_version` disagree (or the impl reports
//! nothing) we paint a "drift" alert badge on that row, plus a
//! `Refresh from spec` action that POSTs to
//! `/_soland/admin/components/{type}/refresh`. The action follows the
//! same 404-tolerant pattern as the other Stream H' admin overrides —
//! when the backend hasn't wired the route yet the operator sees a
//! "endpoint not yet wired" toast rather than a generic error.

use dioxus::prelude::*;

use crate::api::components;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::components::{ComponentCriticality, ComponentImplStatus};
use crate::utils::net::error::format_optional_endpoint_error;

#[component]
pub fn ComponentsPage() -> Element {
    let mut data = use_resource(|| async { components::list_components().await });
    // Per-row in-flight flag keyed by component_type.
    let mut in_flight = use_signal::<Option<String>>(|| None);

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Component registry".to_string(),
                description: "Server-wide component_type / cell_family registry with criticality and version drift indicators.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    "Refresh"
                }
            }

            match &*data.read() {
                Some(Ok(entries)) => {
                    if entries.is_empty() {
                        rsx! {
                            EmptyState {
                                icon: "package".to_string(),
                                title: "No components registered".to_string(),
                                description: "Server returned an empty registry — no ck.component.* types loaded.".to_string(),
                            }
                        }
                    } else {
                        let drift_count = entries.iter().filter(|e| e.has_drift()).count();
                        rsx! {
                            if drift_count > 0 {
                                div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                                    {format!(
                                        "{} component(s) report version drift between spec and impl.",
                                        drift_count
                                    )}
                                }
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { "Component" }
                                            TableHead { "Cell family" }
                                            TableHead { "Criticality" }
                                            TableHead { "Spec" }
                                            TableHead { "Impl" }
                                            TableHead { "Status" }
                                            TableHead { "Drift" }
                                            TableHead { class: "text-right".to_string(), "Action" }
                                        }
                                    }
                                    TableBody {
                                        for e in entries.iter() {
                                            {
                                                let component_type = e.component_type.clone();
                                                let component_type_for_btn = component_type.clone();
                                                let cell_family = e.cell_family.clone();
                                                let crit = e.criticality_typed();
                                                let crit_label = crit.label().to_string();
                                                let crit_variant = criticality_variant(&crit);
                                                let spec = e.spec_version.clone();
                                                let impl_v = if e.impl_version.is_empty() {
                                                    "-".to_string()
                                                } else {
                                                    e.impl_version.clone()
                                                };
                                                let status_typed = e.status_typed();
                                                let status_label = status_typed.label().to_string();
                                                let status_variant = status_variant(&status_typed);
                                                let drift = e.has_drift();
                                                let row_in_flight = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|s| s == component_type)
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs".to_string(), "{component_type}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{cell_family}" }
                                                        TableCell {
                                                            Badge { variant: crit_variant, "{crit_label}" }
                                                        }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{spec}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{impl_v}" }
                                                        TableCell {
                                                            Badge { variant: status_variant, "{status_label}" }
                                                        }
                                                        TableCell {
                                                            if drift {
                                                                Badge { variant: BadgeVariant::Destructive, "Drift" }
                                                            } else {
                                                                Badge { variant: BadgeVariant::Success, "OK" }
                                                            }
                                                        }
                                                        TableCell { class: "text-right".to_string(),
                                                            if drift {
                                                                Button {
                                                                    variant: ButtonVariant::Outline,
                                                                    size: ButtonSize::Sm,
                                                                    disabled: row_in_flight,
                                                                    onclick: move |_| {
                                                                        let ctype = component_type_for_btn.clone();
                                                                        in_flight.set(Some(ctype.clone()));
                                                                        spawn(async move {
                                                                            let res = components::refresh(&ctype).await;
                                                                            match res {
                                                                                Ok(r) => {
                                                                                    let status = r
                                                                                        .status
                                                                                        .clone()
                                                                                        .unwrap_or_else(|| "ok".to_string());
                                                                                    show_toast(
                                                                                        &format!(
                                                                                            "Refreshed {}: {} → {} ({})",
                                                                                            r.component_type,
                                                                                            r.spec_version,
                                                                                            r.impl_version,
                                                                                            status
                                                                                        ),
                                                                                        ToastVariant::Success,
                                                                                    );
                                                                                }
                                                                                Err(e) => {
                                                                                    let msg = format_optional_endpoint_error(
                                                                                        "components refresh",
                                                                                        &e,
                                                                                    );
                                                                                    show_toast(&msg, ToastVariant::Error);
                                                                                }
                                                                            }
                                                                            in_flight.set(None);
                                                                            data.restart();
                                                                        });
                                                                    },
                                                                    "Refresh from spec"
                                                                }
                                                            } else {
                                                                span { class: "text-xs text-muted-foreground", "—" }
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

/// Map criticality to a badge tone: critical = destructive (urgent),
/// important = secondary (notice), optional = outline (passive).
pub(crate) fn criticality_variant(c: &ComponentCriticality) -> BadgeVariant {
    match c {
        ComponentCriticality::Critical => BadgeVariant::Destructive,
        ComponentCriticality::Important => BadgeVariant::Secondary,
        ComponentCriticality::Optional => BadgeVariant::Outline,
    }
}

/// Status badge tone: active = success, stub = secondary (neutral
/// implementation placeholder), disabled = destructive (operator turned it off,
/// admin needs to know).
pub(crate) fn status_variant(s: &ComponentImplStatus) -> BadgeVariant {
    match s {
        ComponentImplStatus::Active => BadgeVariant::Success,
        ComponentImplStatus::Stub => BadgeVariant::Secondary,
        ComponentImplStatus::Disabled => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::{criticality_variant, status_variant};
    use crate::components::ui::badge::BadgeVariant;
    use crate::types::components::{ComponentCriticality, ComponentImplStatus};

    #[test]
    fn criticality_variants_match_severity() {
        assert!(matches!(
            criticality_variant(&ComponentCriticality::Critical),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            criticality_variant(&ComponentCriticality::Important),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            criticality_variant(&ComponentCriticality::Optional),
            BadgeVariant::Outline
        ));
    }

    #[test]
    fn status_variants_match_lifecycle() {
        assert!(matches!(
            status_variant(&ComponentImplStatus::Active),
            BadgeVariant::Success
        ));
        assert!(matches!(
            status_variant(&ComponentImplStatus::Stub),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            status_variant(&ComponentImplStatus::Disabled),
            BadgeVariant::Destructive
        ));
    }
}
