//! Soland authz capability admin page
//!
//! Cursor-paginated list of capability grants visible to the current
//! admin scope. Filterable by holder DID / peer DID / scope (each a
//! case-insensitive substring match). Each row exposes a Revoke button
//! on `Active` rows only — `Revoked` and `Expired` rows render the
//! placeholder em-dash.
//!
//! Follows the 404-tolerant pattern shared with the rest of Stream H' —
//! when the soland route hasn't been wired yet the operator sees a
//! "endpoint not yet wired" toast rather than a generic error.

use dioxus::prelude::*;

use crate::api::authz_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::authz::{AuthzGrantFilter, AuthzGrantStatus, filter_grants};
use crate::utils::net::error::format_optional_endpoint_error;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AuthzCapabilitiesPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut holder_filter = use_signal(String::new);
    let mut peer_filter = use_signal(String::new);
    let mut scope_filter = use_signal(String::new);
    let mut pending_revoke = use_signal::<Option<String>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let holder_snapshot = holder_filter.read().clone();
    let peer_snapshot = peer_filter.read().clone();
    let scope_snapshot = scope_filter.read().clone();

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        let holder = holder_snapshot.clone();
        let peer = peer_snapshot.clone();
        let scope = scope_snapshot.clone();
        async move {
            let f = AuthzGrantFilter {
                holder,
                peer,
                scope,
            };
            authz_admin::list_capability_grants(cursor.as_deref(), PAGE_SIZE, &f).await
        }
    });

    let mut reset_to_first_page = move || {
        cursor_stack.set(vec![None]);
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("authz_caps.title"),
                description: t("authz_caps.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "rounded-lg border p-4 space-y-3",
                div { class: "grid gap-3 md:grid-cols-3",
                    div { class: "space-y-1",
                        Label { r#for: "authz-holder-filter".to_string(), {t("authz_caps.filter_holder")} }
                        Input {
                            value: holder_filter.read().clone(),
                            placeholder: "did:ck:...".to_string(),
                            oninput: move |evt: FormEvent| {
                                reset_to_first_page();
                                holder_filter.set(evt.value());
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { r#for: "authz-peer-filter".to_string(), {t("authz_caps.filter_peer")} }
                        Input {
                            value: peer_filter.read().clone(),
                            placeholder: "did:ck:...".to_string(),
                            oninput: move |evt: FormEvent| {
                                reset_to_first_page();
                                peer_filter.set(evt.value());
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { r#for: "authz-scope-filter".to_string(), {t("authz_caps.filter_scope")} }
                        Input {
                            value: scope_filter.read().clone(),
                            placeholder: "cx.cell....".to_string(),
                            oninput: move |evt: FormEvent| {
                                reset_to_first_page();
                                scope_filter.set(evt.value());
                            },
                        }
                    }
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let server_filter = AuthzGrantFilter {
                        holder: holder_filter.read().clone(),
                        peer: peer_filter.read().clone(),
                        scope: scope_filter.read().clone(),
                    };
                    let rows = filter_grants(&page.data, &server_filter);
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let total_label = match page.total {
                        Some(n) => format!("{n}"),
                        None => "?".to_string(),
                    };
                    let shown = rows.len();
                    rsx! {
                        if rows.is_empty() {
                            EmptyState {
                                icon: "shield".to_string(),
                                title: t("authz_caps.empty_title"),
                                description: t("authz_caps.empty_subtitle"),
                            }
                        } else {
                            p { class: "text-xs text-muted-foreground",
                                {format!("Showing {shown} (server total: {total_label})")}
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("authz_caps.holder")} }
                                            TableHead { {t("authz_caps.peer")} }
                                            TableHead { {t("authz_caps.scope")} }
                                            TableHead { {t("authz_caps.status")} }
                                            TableHead { {t("authz_caps.granted_at")} }
                                            TableHead { {t("authz_caps.expires_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for g in rows.iter() {
                                            {
                                                let grant_id = g.grant_id.clone();
                                                let holder = g.holder_did.clone();
                                                let peer = g.peer_did.clone();
                                                let scope = g.scope.clone();
                                                let granted = g.granted_at.clone().unwrap_or_else(|| "-".to_string());
                                                let expires = g.expires_at.clone().unwrap_or_else(|| "-".to_string());
                                                let typed = g.status_typed();
                                                let variant = grant_status_variant(&typed);
                                                let label = typed.label().to_string();
                                                let revocable = g.is_revocable();
                                                let row_in_flight = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|id| id == grant_id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{holder}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{peer}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{scope}" }
                                                        TableCell {
                                                            Badge { variant, "{label}" }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{granted}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{expires}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            if revocable {
                                                                {
                                                                    let gid = grant_id.clone();
                                                                    rsx! {
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            size: ButtonSize::Sm,
                                                                            disabled: row_in_flight,
                                                                            onclick: move |_| pending_revoke.set(Some(gid.clone())),
                                                                            {t("authz_caps.revoke")}
                                                                        }
                                                                    }
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

                            div { class: "flex items-center justify-between px-2 py-4",
                                div { class: "text-sm text-muted-foreground",
                                    {format!("Page {}", stack_depth)}
                                }
                                div { class: "flex items-center space-x-2",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        disabled: stack_depth <= 1,
                                        onclick: move |_| {
                                            let mut new_stack = cursor_stack.read().clone();
                                            if new_stack.len() > 1 {
                                                new_stack.pop();
                                                cursor_stack.set(new_stack);
                                            }
                                        },
                                        {t("common.previous")}
                                    }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        disabled: next_cursor.is_none(),
                                        onclick: move |_| {
                                            if let Some(c) = next_cursor.clone() {
                                                let mut new_stack = cursor_stack.read().clone();
                                                new_stack.push(Some(c));
                                                cursor_stack.set(new_stack);
                                            }
                                        },
                                        {t("common.next")}
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

            ConfirmDialog {
                open: pending_revoke.read().is_some(),
                title: t("authz_caps.revoke_confirm_title"),
                description: t("authz_caps.revoke_confirm_body"),
                confirm_text: t("authz_caps.revoke"),
                cancel_text: t("common.cancel"),
                destructive: true,
                on_cancel: move |_| pending_revoke.set(None),
                on_confirm: move |_| {
                    if let Some(gid) = pending_revoke.read().clone() {
                        in_flight.set(Some(gid.clone()));
                        spawn(async move {
                            let res = authz_admin::revoke_capability_grant(&gid).await;
                            match res {
                                Ok(_) => show_toast(
                                    "Capability revoked.",
                                    ToastVariant::Success,
                                ),
                                Err(e) => {
                                    let msg = format_optional_endpoint_error(
                                        "authz capability revoke",
                                        &e,
                                    );
                                    show_toast(&msg, ToastVariant::Error);
                                }
                            }
                            in_flight.set(None);
                            data.restart();
                        });
                    }
                    pending_revoke.set(None);
                },
            }
        }
    }
}

/// Pick a Badge variant for a capability grant's status. `Active` is
/// success-green (the happy path); `Revoked` and `Expired` are
/// destructive (red). Pure helper so the mapping is unit-testable.
pub(crate) fn grant_status_variant(status: &AuthzGrantStatus) -> BadgeVariant {
    match status {
        AuthzGrantStatus::Active => BadgeVariant::Success,
        AuthzGrantStatus::Revoked => BadgeVariant::Destructive,
        AuthzGrantStatus::Expired => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::authz::AuthzCapabilityGrant;

    #[test]
    fn status_variant_buckets_match_severity() {
        assert!(matches!(
            grant_status_variant(&AuthzGrantStatus::Active),
            BadgeVariant::Success
        ));
        assert!(matches!(
            grant_status_variant(&AuthzGrantStatus::Revoked),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            grant_status_variant(&AuthzGrantStatus::Expired),
            BadgeVariant::Destructive
        ));
    }

    #[test]
    fn revocable_flag_drives_button_visibility_only_for_active() {
        let g = AuthzCapabilityGrant {
            grant_id: "g1".into(),
            status: "active".into(),
            ..Default::default()
        };
        assert!(g.is_revocable());

        let g = AuthzCapabilityGrant {
            grant_id: "g1".into(),
            status: "revoked".into(),
            ..Default::default()
        };
        assert!(!g.is_revocable());
    }
}
