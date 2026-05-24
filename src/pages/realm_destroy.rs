//! Round R2/R3 — Realm destroy page (T07).
//!
//! Wraps `components::realm_destroy_dialog::RealmDestroyDialog` in a
//! route shell so an operator can navigate to `/realms/{id}/destroy`,
//! tick the five normative bullets, type `DESTROY`, and trigger the
//! `cx.realm.destroy` anchor. After confirmation this page also reuses
//! the 7-domain fanout panel + erasure-receipt block to surface the
//! post-destroy cascade.

use dioxus::prelude::*;

use crate::api::server;
use crate::components::deactivation_fanout_panel::{
    DeactivationFanoutPanel, ErasureReceiptStatus, FanoutDomain, FanoutState, placeholder_snapshot,
};
use crate::components::realm_destroy_dialog::RealmDestroyDialog;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};

#[component]
pub fn RealmDestroyPage(realm_id: String) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut destroyed = use_signal(|| false);

    let realm_id_for_dialog = realm_id.clone();
    let realm_id_for_panel = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Destroy Realm".to_string(),
                description: format!("Irreversible — confirms five normative bullets + the literal `DESTROY` phrase. Round R2/R3 T07. Realm: {realm_id}"),
                Button {
                    variant: ButtonVariant::Destructive,
                    onclick: move |_| dialog_open.set(true),
                    "Open destroy dialog"
                }
            }

            if *destroyed.read() {
                {
                    let mut snap = placeholder_snapshot(realm_id_for_panel.clone());
                    // For `cx.realm.destroy` the panel renders the
                    // erasure-receipt block as well. The placeholder
                    // stays local-only until soland returns peer evidence.
                    snap.erasure_receipt = Some(ErasureReceiptStatus {
                        receipt_id: "receipt:01904100-0000-7000-8000-0000000000ff".into(),
                        local_state: FanoutState::Succeeded,
                        cross_ps_state: None,
                        cross_ps_peers: Vec::new(),
                    });
                    rsx! {
                        DeactivationFanoutPanel {
                            snapshot: snap,
                            on_retry: move |domain: FanoutDomain| {
                                let realm_id = realm_id_for_panel.clone();
                                spawn(async move {
                                    let body = server::RetryRealmDestroyRequest {
                                        domain: domain.slug().to_string(),
                                    };
                                    match server::retry_realm_destroy(&realm_id, &body).await {
                                        Ok(_) => show_toast(
                                            &format!("Retry queued for `{}`.", domain.slug()),
                                            ToastVariant::Success,
                                        ),
                                        Err(err) => show_toast(
                                            &format!("Retry failed: {err}"),
                                            ToastVariant::Error,
                                        ),
                                    }
                                });
                            }
                        }
                    }
                }
            } else {
                p { class: "text-sm text-muted-foreground",
                    "Open the destroy dialog to begin. The 7-domain fanout + erasure receipt panel appears after the anchor lands."
                }
            }

            RealmDestroyDialog {
                open: *dialog_open.read(),
                realm_id: realm_id_for_dialog,
                on_cancel: move |_| dialog_open.set(false),
                on_confirm: move |_| {
                    let realm_id = realm_id.clone();
                    spawn(async move {
                        let body = server::DestroyRealmRequest {
                            confirmation: "DESTROY".to_string(),
                        };
                        match server::destroy_realm(&realm_id, &body).await {
                            Ok(_) => {
                                dialog_open.set(false);
                                destroyed.set(true);
                                show_toast("cx.realm.destroy queued.", ToastVariant::Success);
                            }
                            Err(err) => show_toast(
                                &format!("Realm destroy failed: {err}"),
                                ToastVariant::Error,
                            ),
                        }
                    });
                },
            }
        }
    }
}
