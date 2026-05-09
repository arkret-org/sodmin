//! Consent admin panel (Stream H', H'5).
//!
//! Read-only view over the consent cell or-set values exposed by the
//! soland describe endpoint `GET /api/admin/v1/spaces/{id}/consent`.
//! The page lists one row per (holder, peer, scope) triple, with a
//! holder-DID filter to narrow down by user. Admins cannot grant or
//! revoke from this page — the consent capability is bound to the
//! holder's signing key, so this surface is intentionally read-only.

use dioxus::prelude::*;

use crate::api::consent_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::consent::{ConsentStatus, filter_by_holder};

#[component]
pub fn ConsentPage(space_id: String) -> Element {
    let space_id_for_fetch = space_id.clone();
    let mut data = use_resource(move || {
        let id = space_id_for_fetch.clone();
        async move { consent_admin::list_consent_grants(&id).await }
    });

    let mut holder_filter = use_signal(String::new);
    let header_space_id = space_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Consent · {}", header_space_id),
                description: "Read-only view of consent cell or-set values inside this Space.".to_string(),
            }

            div { class: "max-w-md space-y-1",
                Label { r#for: "consent-holder-filter".to_string(), "Filter by holder DID" }
                Input {
                    value: holder_filter.read().clone(),
                    placeholder: "did:cx:...".to_string(),
                    oninput: move |evt: FormEvent| holder_filter.set(evt.value()),
                }
            }

            match &*data.read() {
                Some(Ok(all_grants)) => {
                    let filter_text = holder_filter.read().clone();
                    let filtered = filter_by_holder(all_grants, &filter_text);
                    let total = all_grants.len();
                    let shown = filtered.len();
                    rsx! {
                        p { class: "text-xs text-muted-foreground",
                            {format!("Showing {shown} of {total} grants.")}
                        }
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { "Holder DID" }
                                        TableHead { "Peer DID" }
                                        TableHead { "Scope" }
                                        TableHead { "Status" }
                                        TableHead { "Created" }
                                    }
                                }
                                TableBody {
                                    if filtered.is_empty() {
                                        TableRow {
                                            TableCell {
                                                class: "text-center text-muted-foreground py-8".to_string(),
                                                colspan: 99,
                                                "No consent grants match the current filter."
                                            }
                                        }
                                    } else {
                                        for g in filtered.iter() {
                                            {
                                                let holder = g.holder_did.clone();
                                                let peer = g.peer_did.clone();
                                                let scope = g.scope.clone();
                                                let created = g.created_at.clone().unwrap_or_else(|| "-".to_string());
                                                let typed = g.status_typed();
                                                let variant = consent_status_variant(&typed);
                                                let label = typed.label().to_string();
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[280px] truncate".to_string(), "{holder}" }
                                                        TableCell { class: "font-mono text-xs max-w-[280px] truncate".to_string(), "{peer}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{scope}" }
                                                        TableCell {
                                                            Badge { variant, "{label}" }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{created}" }
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

/// Pick a Badge variant for a consent status. `Active` is success-green,
/// `Revoked`/`Expired` are destructive, `Pending` is the neutral
/// secondary tone.
pub(crate) fn consent_status_variant(status: &ConsentStatus) -> BadgeVariant {
    match status {
        ConsentStatus::Active => BadgeVariant::Success,
        ConsentStatus::Revoked => BadgeVariant::Destructive,
        ConsentStatus::Expired => BadgeVariant::Destructive,
        ConsentStatus::Pending => BadgeVariant::Secondary,
    }
}

#[cfg(test)]
mod tests {
    use super::consent_status_variant;
    use crate::components::ui::badge::BadgeVariant;
    use crate::types::consent::ConsentStatus;

    #[test]
    fn status_variant_buckets_match_severity() {
        assert!(matches!(
            consent_status_variant(&ConsentStatus::Active),
            BadgeVariant::Success
        ));
        assert!(matches!(
            consent_status_variant(&ConsentStatus::Revoked),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            consent_status_variant(&ConsentStatus::Expired),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            consent_status_variant(&ConsentStatus::Pending),
            BadgeVariant::Secondary
        ));
    }
}
