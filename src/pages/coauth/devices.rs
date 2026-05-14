//! Per-account device admin page
//!
//! Cursor-paginated list of devices registered to a single account.
//! Each row shows status / last-seen / linked session count and (for
//! `Active` / `Stale` rows) a destructive Revoke button.
//!
//! The cascade revoke of session grants on the soland side is wired in
//! coauth
//! soland reacts. Follows the 404-tolerant pattern shared with the rest
//! of Stream H'.

use dioxus::prelude::*;

use crate::api::coauth_devices_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::coauth_devices::{CoauthDeviceStatus, render_last_seen};
use crate::utils::error::format_optional_endpoint_error;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AccountDevicesPage(account_id: String) -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut pending_revoke = use_signal::<Option<String>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let account_for_fetch = account_id.clone();

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        let acct = account_for_fetch.clone();
        async move {
            coauth_devices_admin::list_account_devices(&acct, cursor.as_deref(), PAGE_SIZE).await
        }
    });

    let header_account_id = account_id.clone();
    let account_for_revoke = account_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("{} · {}", t("coauth_devices.title"), header_account_id),
                description: t("coauth_devices.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let total_label = match page.total {
                        Some(n) => format!("{n}"),
                        None => "?".to_string(),
                    };
                    let row_count = page.data.len();
                    rsx! {
                        if page.data.is_empty() {
                            EmptyState {
                                icon: "smartphone".to_string(),
                                title: t("coauth_devices.empty_title"),
                                description: t("coauth_devices.empty_subtitle"),
                            }
                        } else {
                            p { class: "text-xs text-muted-foreground",
                                {format!("Showing {row_count} (server total: {total_label})")}
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("coauth_devices.device_id")} }
                                            TableHead { {t("coauth_devices.display_name")} }
                                            TableHead { {t("coauth_devices.status")} }
                                            TableHead { {t("coauth_devices.last_seen")} }
                                            TableHead { {t("coauth_devices.linked_sessions")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for row in page.data.iter() {
                                            {
                                                let device_id = row.device_id.clone();
                                                let display_name = row
                                                    .display_name
                                                    .clone()
                                                    .unwrap_or_else(|| "-".to_string());
                                                let last_seen = render_last_seen(row);
                                                let linked = row.linked_session_count;
                                                let typed = row.status_typed();
                                                let label = typed.label().to_string();
                                                let variant = device_status_variant(&typed);
                                                let revocable = row.is_revocable();
                                                let row_in_flight = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|id| id == device_id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{device_id}" }
                                                        TableCell { "{display_name}" }
                                                        TableCell {
                                                            Badge { variant, "{label}" }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{last_seen}" }
                                                        TableCell { "{linked}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            if revocable {
                                                                {
                                                                    let did = device_id.clone();
                                                                    rsx! {
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            size: ButtonSize::Sm,
                                                                            disabled: row_in_flight,
                                                                            onclick: move |_| pending_revoke.set(Some(did.clone())),
                                                                            {t("coauth_devices.revoke")}
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
                title: t("coauth_devices.revoke_confirm_title"),
                description: t("coauth_devices.revoke_confirm_body"),
                confirm_text: t("coauth_devices.revoke"),
                cancel_text: t("common.cancel"),
                destructive: true,
                on_cancel: move |_| pending_revoke.set(None),
                on_confirm: move |_| {
                    if let Some(did) = pending_revoke.read().clone() {
                        in_flight.set(Some(did.clone()));
                        let acct = account_for_revoke.clone();
                        spawn(async move {
                            let res = coauth_devices_admin::revoke_account_device(
                                &acct, &did,
                            )
                            .await;
                            match res {
                                Ok(_) => show_toast(
                                    "Device revoked. Linked session grants cascade-revoke on soland side.",
                                    ToastVariant::Success,
                                ),
                                Err(e) => {
                                    let msg = format_optional_endpoint_error(
                                        "device revoke",
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

/// Pick a Badge variant for a device status. `Active` is success-green;
/// `Stale` is the neutral secondary tone (still revocable but not
/// healthy); `Revoked` is destructive (red). Pure helper.
pub(crate) fn device_status_variant(status: &CoauthDeviceStatus) -> BadgeVariant {
    match status {
        CoauthDeviceStatus::Active => BadgeVariant::Success,
        CoauthDeviceStatus::Stale => BadgeVariant::Secondary,
        CoauthDeviceStatus::Revoked => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coauth_devices::CoauthDeviceRow;

    #[test]
    fn device_status_variant_buckets_match_severity() {
        assert!(matches!(
            device_status_variant(&CoauthDeviceStatus::Active),
            BadgeVariant::Success
        ));
        assert!(matches!(
            device_status_variant(&CoauthDeviceStatus::Stale),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            device_status_variant(&CoauthDeviceStatus::Revoked),
            BadgeVariant::Destructive
        ));
    }

    #[test]
    fn revoke_button_visible_only_when_row_revocable() {
        // Stale and Active rows expose the revoke button; Revoked rows
        // do not.
        let r = CoauthDeviceRow {
            device_id: "d1".into(),
            status: "active".into(),
            ..Default::default()
        };
        assert!(r.is_revocable());
        let r = CoauthDeviceRow {
            device_id: "d1".into(),
            status: "stale".into(),
            ..Default::default()
        };
        assert!(r.is_revocable());
        let r = CoauthDeviceRow {
            device_id: "d1".into(),
            status: "revoked".into(),
            ..Default::default()
        };
        assert!(!r.is_revocable());
    }
}
