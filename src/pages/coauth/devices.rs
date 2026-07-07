//! Per-account device admin page
//!
//! List of devices registered to a single account. Each row shows the current
//! coauth device record and exposes a destructive Revoke button until coauth
//! reports a `revoked_at` timestamp.

use dioxus::prelude::*;

use crate::api::coauth_devices;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::modal::{DialogActions, Modal};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::coauth_devices::{
    CoauthDeviceMfaState, CoauthDeviceRiskLevel, render_optional_timestamp,
};
use crate::utils::i18n::t;
use crate::utils::net::error::format_optional_endpoint_error;

#[component]
pub fn AccountDevicesPage(account_id: String) -> Element {
    let mut pending_revoke = use_signal::<Option<String>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);
    let mut revoke_reason = use_signal(String::new);

    let account_for_fetch = account_id.clone();

    let mut data = use_resource(move || {
        let acct = account_for_fetch.clone();
        async move { coauth_devices::list_account_devices(&acct).await }
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
                Some(Ok(rows)) => {
                    rsx! {
                        if rows.is_empty() {
                            EmptyState {
                                icon_name: "smartphone".to_string(),
                                title: t("coauth_devices.empty_title"),
                                description: t("coauth_devices.empty_subtitle"),
                            }
                        } else {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("coauth_devices.device_id")} }
                                            TableHead { {t("coauth_devices.display_name")} }
                                            TableHead { {t("coauth_devices.risk_level")} }
                                            TableHead { {t("coauth_devices.mfa_state")} }
                                            TableHead { {t("coauth_devices.registered_at")} }
                                            TableHead { {t("coauth_devices.revoked_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for row in rows.iter() {
                                            {
                                                let device_id = row.id.clone();
                                                let display_name = row
                                                    .display_name
                                                    .clone()
                                                    .unwrap_or_else(|| "-".to_string());
                                                let risk_label = row.risk_level.label().to_string();
                                                let risk_variant = device_risk_variant(&row.risk_level);
                                                let mfa_label = row.mfa_state.label().to_string();
                                                let mfa_variant = device_mfa_variant(&row.mfa_state);
                                                let registered_at = render_optional_timestamp(row.registered_at.as_deref());
                                                let revoked_at = render_optional_timestamp(row.revoked_at.as_deref());
                                                let revocable = row.is_revocable();
                                                let row_in_flight = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|id| id == device_id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        key: "{device_id}",
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{device_id}" }
                                                        TableCell { "{display_name}" }
                                                        TableCell {
                                                            Badge { variant: risk_variant, "{risk_label}" }
                                                        }
                                                        TableCell {
                                                            Badge { variant: mfa_variant, "{mfa_label}" }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{registered_at}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{revoked_at}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            if revocable {
                                                                {
                                                                    let did = device_id.clone();
                                                                    rsx! {
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            size: ButtonSize::Sm,
                                                                            disabled: row_in_flight,
                                                                            onclick: move |_| {
                                                                                revoke_reason.set(String::new());
                                                                                pending_revoke.set(Some(did.clone()));
                                                                            },
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

            Modal {
                open: pending_revoke.read().is_some(),
                title: t("coauth_devices.revoke_confirm_title"),
                on_close: move |_| pending_revoke.set(None),
                p { class: "text-sm text-muted-foreground", {t("coauth_devices.revoke_confirm_body")} }
                div { class: "space-y-1",
                    Label { r#for: "coauth-device-revoke-reason".to_string(), "Reason / ticket" }
                    Input {
                        id: "coauth-device-revoke-reason".to_string(),
                        value: revoke_reason.read().clone(),
                        required: true,
                        placeholder: "SEC-1234 / support case / incident reason".to_string(),
                        oninput: move |evt: FormEvent| revoke_reason.set(evt.value()),
                    }
                }
                {
                    let reason_ready = !revoke_reason.read().trim().is_empty();
                    let busy = in_flight.read().is_some();
                    rsx! {
                        DialogActions {
                            confirm_text: t("coauth_devices.revoke"),
                            cancel_text: t("common.cancel"),
                            destructive: true,
                            confirm_loading: busy || !reason_ready,
                            on_cancel: move |_| pending_revoke.set(None),
                            on_confirm: move |_| {
                                let reason = revoke_reason.read().trim().to_string();
                                if reason.is_empty() {
                                    return;
                                }
                                if let Some(did) = pending_revoke.read().clone() {
                                    in_flight.set(Some(did.clone()));
                                    let acct = account_for_revoke.clone();
                                    spawn(async move {
                                        let res = coauth_devices::revoke_account_device(
                                            &acct,
                                            &did,
                                            &reason,
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
        }
    }
}

pub(crate) fn device_risk_variant(risk: &CoauthDeviceRiskLevel) -> BadgeVariant {
    match risk {
        CoauthDeviceRiskLevel::Low => BadgeVariant::Success,
        CoauthDeviceRiskLevel::Medium => BadgeVariant::Secondary,
        CoauthDeviceRiskLevel::High => BadgeVariant::Destructive,
        CoauthDeviceRiskLevel::Unknown => BadgeVariant::Outline,
    }
}

pub(crate) fn device_mfa_variant(state: &CoauthDeviceMfaState) -> BadgeVariant {
    match state {
        CoauthDeviceMfaState::Verified => BadgeVariant::Success,
        CoauthDeviceMfaState::Required => BadgeVariant::Destructive,
        CoauthDeviceMfaState::Unknown => BadgeVariant::Outline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coauth_devices::CoauthDeviceRow;

    #[test]
    fn device_risk_variant_buckets_match_severity() {
        assert!(matches!(
            device_risk_variant(&CoauthDeviceRiskLevel::Low),
            BadgeVariant::Success
        ));
        assert!(matches!(
            device_risk_variant(&CoauthDeviceRiskLevel::Medium),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            device_risk_variant(&CoauthDeviceRiskLevel::High),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            device_risk_variant(&CoauthDeviceRiskLevel::Unknown),
            BadgeVariant::Outline
        ));
    }

    #[test]
    fn device_mfa_variant_buckets_match_state() {
        assert!(matches!(
            device_mfa_variant(&CoauthDeviceMfaState::Verified),
            BadgeVariant::Success
        ));
        assert!(matches!(
            device_mfa_variant(&CoauthDeviceMfaState::Required),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            device_mfa_variant(&CoauthDeviceMfaState::Unknown),
            BadgeVariant::Outline
        ));
    }

    #[test]
    fn revoke_button_visible_only_when_row_revocable() {
        let r = CoauthDeviceRow {
            id: "d1".into(),
            account_id: None,
            display_name: None,
            risk_level: CoauthDeviceRiskLevel::Unknown,
            mfa_state: CoauthDeviceMfaState::Unknown,
            registered_at: None,
            revoked_at: None,
        };
        assert!(r.is_revocable());
        let r = CoauthDeviceRow {
            revoked_at: Some("2026-05-09T12:00:00Z".into()),
            ..r
        };
        assert!(!r.is_revocable());
    }
}
