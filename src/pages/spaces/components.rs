//! Component criticality / version drift admin page (Stream H', H'6).
//!
//! Server-wide component registry view. Reads from
//! `GET /api/admin/v1/components` and renders one row per
//! `cx.component.*` type with the cell_family it projects into, the
//! criticality classification, the spec version pinned by the bundle, the
//! impl version actually loaded, and an Active/Stub/Disabled status.
//! When `spec_version` and `impl_version` disagree (or the impl reports
//! nothing) we paint a "drift" alert badge on that row.

use dioxus::prelude::*;

use crate::api::components_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::components::{ComponentCriticality, ComponentImplStatus};

#[component]
pub fn ComponentsPage() -> Element {
    let mut data = use_resource(|| async { components_admin::list_components().await });

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
                                    }
                                }
                                TableBody {
                                    if entries.is_empty() {
                                        TableRow {
                                            TableCell {
                                                class: "text-center text-muted-foreground py-8".to_string(),
                                                colspan: 99,
                                                "Server reported no components."
                                            }
                                        }
                                    } else {
                                        for e in entries.iter() {
                                            {
                                                let component_type = e.component_type.clone();
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
/// "scaffold present"), disabled = destructive (operator turned it off,
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
