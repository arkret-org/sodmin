//! Notary signing-key admin page (Stream H', H'8).
//!
//! Read-mostly view over the NotaryWorker's currently-bound signing key:
//! origin (Configured / Ephemeral), verification method id (`<did>#<kid>`),
//! and last-rotation timestamp. When the key is `Configured` the page
//! exposes a `Rotate signing key` button that POSTs to
//! `/_soland/admin/realms/{id}/notary/rotate-signing-key`. When the key
//! is `Ephemeral` the rotation button is hidden and a destructive banner
//! warns the operator that production deployments must redeploy with a
//! configured key (rotating an ephemeral key just spawns another
//! ephemeral key, leaving notary signatures without stable DID binding).
//!
//! Follows the 404-tolerant pattern shared with the rest of Stream H' —
//! when the soland route hasn't been wired the operator sees a clear
//! "endpoint not yet wired" toast rather than a generic error.

use dioxus::prelude::*;

use crate::api::signing_key;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::signing_key::SigningKeyOrigin;
use crate::utils::net::error::format_optional_endpoint_error;

#[component]
pub fn SigningKeysPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { signing_key::get_signing_key(&id).await }
    });

    let mut confirming = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let header_realm_id = realm_id.clone();
    let realm_id_for_rotate = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Signing key · {}", header_realm_id),
                description: "NotaryWorker signing key origin, verification method and rotation controls.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    "Refresh"
                }
            }

            match &*data.read() {
                Some(Ok(describe)) => {
                    let origin_typed = describe.origin_typed();
                    let origin_label = origin_typed.label().to_string();
                    let origin_variant = origin_badge_variant(&origin_typed);
                    let vm = describe.verification_method_id.clone();
                    let did = describe.did.clone().unwrap_or_else(|| "-".to_string());
                    let kid = describe.kid.clone().unwrap_or_else(|| "-".to_string());
                    let alg = describe.algorithm.clone().unwrap_or_else(|| "-".to_string());
                    let last_rotated = describe
                        .last_rotated_at
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let can_rotate = describe.can_rotate();
                    let is_unverified_origin = !matches!(origin_typed, SigningKeyOrigin::Configured);
                    rsx! {
                        if is_unverified_origin {
                            // Production-readiness warning: ephemeral keys
                            // disappear on restart and are NEVER suitable
                            // for live workloads. The destructive banner
                            // mirrors the covered_seals lag pattern.
                            div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                                "NotaryWorker signing-key origin is not confirmed by a backend describe endpoint. Rotation is disabled until soland exposes authoritative key origin and persistence metadata."
                            }
                        }
                        Card {
                            CardHeader { CardTitle { "Current signing key" } }
                            CardContent {
                                div { class: "space-y-3 text-sm",
                                    div { class: "flex items-center gap-2",
                                        Badge { variant: origin_variant, "{origin_label}" }
                                        if is_unverified_origin {
                                            span { class: "text-xs text-muted-foreground",
                                                "(read-only until key origin is confirmed)"
                                            }
                                        }
                                    }
                                    div {
                                        span { class: "text-muted-foreground mr-2", "Verification method:" }
                                        span { class: "font-mono text-xs break-all", "{vm}" }
                                    }
                                    div { class: "grid grid-cols-2 gap-3",
                                        div {
                                            span { class: "text-muted-foreground mr-2", "DID:" }
                                            span { class: "font-mono text-xs break-all", "{did}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "kid:" }
                                            span { class: "font-mono text-xs break-all", "{kid}" }
                                        }
                                    }
                                    div { class: "grid grid-cols-2 gap-3",
                                        div {
                                            span { class: "text-muted-foreground mr-2", "Algorithm:" }
                                            span { class: "font-mono text-xs", "{alg}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "Last rotated:" }
                                            span { class: "font-mono text-xs", "{last_rotated}" }
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { "Rotation" } }
                            CardContent {
                                div { class: "flex items-center justify-between",
                                    p { class: "text-sm text-muted-foreground max-w-2xl",
                                        if can_rotate {
                                            "Rotating generates a fresh key, swaps the worker's signer atomically, and reports the new verification method id. Audit-logged."
                                        } else {
                                            "Rotation disabled for this signing-key origin. Configure a stable PEM / KMS-backed signer and redeploy to enable."
                                        }
                                    }
                                    Button {
                                        variant: ButtonVariant::Default,
                                        disabled: !can_rotate || *submitting.read(),
                                        onclick: move |_| confirming.set(true),
                                        "Rotate signing key"
                                    }
                                }
                            }
                        }

                        // Confirmation modal — destructive because rotation
                        // is irreversible (the previous key material is
                        // discarded). The button stays disabled while the
                        // POST is in flight.
                        {
                            let open = *confirming.read();
                            let confirm_text = if *submitting.read() {
                                "Rotating…".to_string()
                            } else {
                                "Confirm rotation".to_string()
                            };
                            let realm_id = realm_id_for_rotate.clone();
                            rsx! {
                                ConfirmDialog {
                                    open,
                                    title: "Rotate NotaryWorker signing key?".to_string(),
                                    description: "This generates a fresh key and rebinds the NotaryWorker. Existing in-flight Seals will be re-signed with the new key. Audit-logged. Continue?".to_string(),
                                    confirm_text,
                                    cancel_text: "Cancel".to_string(),
                                    destructive: true,
                                    on_cancel: move |_| confirming.set(false),
                                    on_confirm: move |_| {
                                        if *submitting.read() { return; }
                                        submitting.set(true);
                                        let id = realm_id.clone();
                                        spawn(async move {
                                            let res = signing_key::rotate_signing_key(&id).await;
                                            match res {
                                                Ok(r) => show_toast(
                                                    &format!(
                                                        "Rotated to {}",
                                                        r.verification_method_id
                                                    ),
                                                    ToastVariant::Success,
                                                ),
                                                Err(e) => {
                                                    let msg = format_optional_endpoint_error(
                                                        "signing-key rotate",
                                                        &e,
                                                    );
                                                    show_toast(&msg, ToastVariant::Error);
                                                }
                                            }
                                            submitting.set(false);
                                            confirming.set(false);
                                            data.restart();
                                        });
                                    },
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

/// Map signing-key origin to a Badge variant: `Configured` is success
/// (green; happy path), `Ephemeral` is destructive (red; production
/// hazard). Pure helper so the mapping is unit-testable independent of
/// the page.
pub(crate) fn origin_badge_variant(origin: &SigningKeyOrigin) -> BadgeVariant {
    match origin {
        SigningKeyOrigin::Configured => BadgeVariant::Success,
        SigningKeyOrigin::Ephemeral | SigningKeyOrigin::Unknown => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_variant_tracks_severity() {
        // Configured key is the happy path → green/success.
        assert!(matches!(
            origin_badge_variant(&SigningKeyOrigin::Configured),
            BadgeVariant::Success
        ));
        // Ephemeral key is a production hazard → destructive.
        assert!(matches!(
            origin_badge_variant(&SigningKeyOrigin::Ephemeral),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            origin_badge_variant(&SigningKeyOrigin::Unknown),
            BadgeVariant::Destructive
        ));
    }
}
