//! Round R2/R3 — Realm destroy page (T07).
//!
//! Wraps `components::realm_destroy_dialog::RealmDestroyDialog` in a
//! route shell so an operator can navigate to `/realms/{id}/destroy`,
//! tick the five normative bullets, type `DESTROY`, and trigger the
//! `ak.realm.destroy` Control Move. After confirmation this page also reuses
//! the server acknowledgement.

use dioxus::prelude::*;

use crate::api::server;
use crate::components::realm_destroy_dialog::RealmDestroyDialog;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

#[component]
pub fn DestroyPage(realm_id: String) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut destroyed = use_signal(|| false);

    let realm_id_for_dialog = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_destroy.title"),
                description: t("realm_destroy.description").replace("{realm_id}", &realm_id),
                Button {
                    variant: ButtonVariant::Destructive,
                    onclick: move |_| dialog_open.set(true),
                    {t("realm_destroy.open_dialog")}
                }
            }

            if *destroyed.read() {
                div {
                    class: "rounded-md border border-green-600/40 bg-green-600/10 px-3 py-2 text-sm",
                    role: "status",
                    {t("realm_destroy.completed")}
                }
            } else {
                p { class: "text-sm text-muted-foreground",
                    {t("realm_destroy.intro")}
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
                            Ok(outcome) => {
                                dialog_open.set(false);
                                destroyed.set(true);
                                show_toast(
                                    &t("realm_destroy.toast_queued")
                                        .replace("{realm}", &outcome.realm_id)
                                        .replace("{deleted}", &outcome.deleted.to_string()),
                                    ToastVariant::Success,
                                );
                            }
                            Err(err) => show_toast(
                                &t("realm_destroy.toast_failed").replace("{err}", &err.to_string()),
                                ToastVariant::Error,
                            ),
                        }
                    });
                },
            }
        }
    }
}
