//! Round R2/R3 + Round 4 — Trust domain deployment setting (T08).
//!
//! Edits the deployment-wide `ck:trust_domain:<scope>` value that
//! enters the canonical transcript of every `cx.cross_signing.publish`
//! (and `cx.cross_signing.reset`) proof, plus every federation S2S
//! signing transcript (`Source-Trust-Domain` / `Destination-Trust-Domain`
//! per Round 4 spec a77b995).
//!
//! Changing the trust_domain after first set INVALIDATES every
//! previously-issued cross-signing publish / reset proof — proof bytes
//! from one domain cannot be replayed into another. The page therefore:
//!
//! 1. Shows the current value as read-only.
//! 2. Requires the admin to flip a "re-confirm" toggle before the edit field becomes writable.
//! 3. Validates the new value against the `ck:trust_domain:<lowercase-scope>` grammar before
//!    allowing submit.
//! 4. Surfaces a loud warning callout describing the invalidation.

use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::input::Input;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};

/// The fixed prefix every `ck:trust_domain:*` value MUST carry.
const TRUST_DOMAIN_PREFIX: &str = "ck:trust_domain:";
/// Maximum suffix length the SDK validates against
/// (`TypedTrustDomainId::new` rejects > 128 chars after the prefix).
const TRUST_DOMAIN_MAX_SUFFIX_LEN: usize = 128;

#[component]
pub fn TrustDomainConfigPage() -> Element {
    let mut current = use_signal(String::new);
    let mut draft = use_signal(|| current.read().clone());
    let mut unlocked = use_signal(|| false);
    let mut validation_error = use_signal::<Option<String>>(|| None);
    let mut hydrated = use_signal(|| false);
    let mut setting_data = use_resource(|| async { server::get_trust_domain().await });

    if !*hydrated.read()
        && let Some(Ok(setting)) = setting_data.read().as_ref()
    {
        current.set(setting.value.clone());
        draft.set(setting.value.clone());
        hydrated.set(true);
    }

    let current_value = current.read().clone();
    let draft_value = draft.read().clone();
    let writable = *unlocked.read();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Trust domain".to_string(),
                description: "Edit the deployment-wide `ck:trust_domain:<scope>` value that anchors every cross-signing publish / reset proof and the round-4 federation `Source-Trust-Domain` / `Destination-Trust-Domain` headers.".to_string(),
            }

            // Big warning callout — the entire point of this page is
            // making sure the admin reads this before saving.
            div { class: "rounded-md border border-red-600/40 bg-red-600/10 p-4 text-sm space-y-1",
                role: "alert",
                p { class: "font-semibold text-red-700 dark:text-red-300",
                    "Old cross_signing.publish proofs not replayable across trust_domain change."
                }
                p { class: "text-red-700 dark:text-red-200",
                    "The trust_domain enters the canonical transcript of every `cx.cross_signing.publish` / `cx.cross_signing.reset` proof and every federation S2S signature (round 4: `Source-Trust-Domain` / `Destination-Trust-Domain` / `Request-Canonical-Digest`). Proofs anchored under the previous domain CANNOT be replayed under the new one — they verify to bytes that include the old domain string. Affected principals will need to issue fresh cross-signing keys and re-publish; federated peers must be re-handshaked."
                }
            }

            Card {
                CardHeader {
                    div { class: "flex items-start justify-between gap-2",
                        div { class: "space-y-1",
                            CardTitle { class: "text-lg".to_string(), "Current value" }
                            CardDescription {
                                "Deployment-wide; shared across every Realm on this principal server."
                            }
                        }
                        Badge { variant: BadgeVariant::Secondary, "read-only" }
                    }
                }
                CardContent {
                    p { class: "font-mono text-sm break-all", "{current_value}" }
                }
            }

            Card {
                CardHeader {
                    CardTitle { class: "text-lg".to_string(), "Re-confirm and edit" }
                    CardDescription {
                        "Tick the box below to unlock the input field. The new value must match `ck:trust_domain:<lowercase-scope>` and be \u{2264} 128 chars after the prefix."
                    }
                }
                CardContent { class: "space-y-3".to_string(),
                    label { class: "flex items-start gap-2 text-sm",
                        input {
                            r#type: "checkbox",
                            class: "mt-1",
                            checked: writable,
                            onchange: move |evt| {
                                let parsed = evt.value().parse::<bool>().unwrap_or(!writable);
                                unlocked.set(parsed);
                                if !parsed {
                                    // Snap the draft back to the current
                                    // value if the admin re-locks the
                                    // field — they should not be able to
                                    // sneak a stale draft past the
                                    // re-confirm gate.
                                    draft.set(current.read().clone());
                                    validation_error.set(None);
                                }
                            },
                        }
                        span {
                            "I understand changing this value invalidates every `cx.cross_signing.publish` and `cx.cross_signing.reset` proof anchored to date, and breaks federation signatures that were canonicalised under the previous domain."
                        }
                    }
                    Input {
                        r#type: "text".to_string(),
                        placeholder: "ck:trust_domain:example.net".to_string(),
                        value: draft_value.clone(),
                        disabled: !writable,
                        oninput: move |evt: FormEvent| {
                            validation_error.set(None);
                            draft.set(evt.value());
                        },
                    }
                    if let Some(err) = validation_error.read().as_ref() {
                        p { class: "text-xs text-destructive font-mono", "{err}" }
                    }
                    div { class: "flex justify-end gap-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: !writable,
                            onclick: move |_| {
                                draft.set(current.read().clone());
                                validation_error.set(None);
                            },
                            "Reset to current"
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            disabled: !writable,
                            onclick: move |_| {
                                let new_value = draft.read().trim().to_string();
                                match validate_trust_domain(&new_value) {
                                    Ok(()) => {
                                        if new_value == *current.read() {
                                            validation_error.set(Some(
                                                "new value matches current — nothing to do".into(),
                                            ));
                                            return;
                                        }
                                        spawn(async move {
                                            let body = server::UpdateTrustDomainRequest {
                                                value: new_value,
                                                reconfirm: true,
                                            };
                                            match server::update_trust_domain(&body).await {
                                                Ok(updated) => {
                                                    current.set(updated.value.clone());
                                                    draft.set(updated.value);
                                                    unlocked.set(false);
                                                    validation_error.set(None);
                                                    hydrated.set(false);
                                                    setting_data.restart();
                                                    show_toast("Trust domain updated.", ToastVariant::Success);
                                                }
                                                Err(err) => validation_error.set(Some(format!("{err}"))),
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        validation_error.set(Some(e));
                                    }
                                }
                            },
                            "Apply new trust_domain"
                        }
                    }
                }
            }
        }
    }
}

/// Validate the candidate value against the
/// `ck:trust_domain:<lowercase-scope>` grammar. Mirrors the SDK-side
/// `TypedTrustDomainId::new` check so we surface a useful error before
/// the round-trip.
fn validate_trust_domain(value: &str) -> Result<(), String> {
    if !value.starts_with(TRUST_DOMAIN_PREFIX) {
        return Err(format!("must start with `{TRUST_DOMAIN_PREFIX}`"));
    }
    let suffix = &value[TRUST_DOMAIN_PREFIX.len()..];
    if suffix.is_empty() {
        return Err("scope (after the `ck:trust_domain:` prefix) must not be empty".into());
    }
    if suffix.len() > TRUST_DOMAIN_MAX_SUFFIX_LEN {
        return Err(format!(
            "scope is {} chars, max {TRUST_DOMAIN_MAX_SUFFIX_LEN}",
            suffix.len()
        ));
    }
    // Lowercase alpha-num + `.` / `-` / `_`. The SDK's grammar is
    // stricter — we mirror it here so the admin gets local feedback.
    let ok = suffix
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '.' || c == '-' || c == '_');
    if !ok {
        return Err(
            "scope must be lowercase ASCII alphanum plus `.`, `-`, `_` (no upper-case letters)"
                .into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_well_formed_values() {
        assert!(validate_trust_domain("ck:trust_domain:example.net").is_ok());
        assert!(validate_trust_domain("ck:trust_domain:foo_bar-baz.example").is_ok());
    }

    #[test]
    fn validate_rejects_missing_prefix() {
        assert!(validate_trust_domain("trust_domain:foo").is_err());
        assert!(validate_trust_domain("ck:realm:foo").is_err());
    }

    #[test]
    fn validate_rejects_uppercase_scope() {
        // SDK lower-cases trust_domain scopes; reject early so the
        // admin sees the same error locally.
        assert!(validate_trust_domain("ck:trust_domain:Example").is_err());
    }

    #[test]
    fn validate_rejects_empty_scope() {
        assert!(validate_trust_domain("ck:trust_domain:").is_err());
    }

    #[test]
    fn validate_rejects_oversize_scope() {
        let too_long = format!(
            "ck:trust_domain:{}",
            "a".repeat(TRUST_DOMAIN_MAX_SUFFIX_LEN + 1)
        );
        assert!(validate_trust_domain(&too_long).is_err());
    }
}
