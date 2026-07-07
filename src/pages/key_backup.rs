//! B-C key-backup admin page (`/key-backup`).
//!
//! Three panes:
//!   1. Backup series list with frontier status + per-series 3-class 409 counters
//!      (`series_chain_broken` / `series_seq_not_monotonic` / `series_predecessor_not_found`).
//!   2. Recovery policy history (read-only; spec `recovery_policy_summary` rows — policy publish
//!      requires a principal-signed `auth_data` transcript the admin UI cannot mint).
//!   3. Recovery receipt history.

use cokret_core::models::KeyBackupSummary;
use dioxus::prelude::*;

use crate::api::key_backup;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::RecoveryPolicySummary;
use crate::utils::i18n::t;

#[component]
pub fn KeyBackupList() -> Element {
    let mut series_filter = use_signal(String::new);
    let mut backup_class_filter = use_signal(String::new);
    let mut principal_filter = use_signal(String::new);

    let series_q = series_filter.read().clone();
    let class_q = backup_class_filter.read().clone();
    let mut series_data = use_resource(move || {
        let series_q = series_q.clone();
        let class_q = class_q.clone();
        async move {
            let s = if series_q.is_empty() {
                None
            } else {
                Some(series_q.as_str())
            };
            let c = if class_q.is_empty() {
                None
            } else {
                Some(class_q.as_str())
            };
            // Re-bind to satisfy borrow checker — we own the strings.
            let _ = (s, c);
            let series_owned = series_q.clone();
            let class_owned = class_q.clone();
            let s2 = if series_owned.is_empty() {
                None
            } else {
                Some(series_owned.as_str())
            };
            let c2 = if class_owned.is_empty() {
                None
            } else {
                Some(class_owned.as_str())
            };
            key_backup::list_backups(s2, c2).await
        }
    });

    let principal_q = principal_filter.read().clone();
    let mut policies_data = use_resource(move || {
        let principal_q = principal_q.clone();
        async move {
            let principal_id = if principal_q.is_empty() {
                None
            } else {
                Some(principal_q.as_str())
            };
            key_backup::list_recovery_policies(principal_id).await
        }
    });

    let receipts_principal_q = principal_filter.read().clone();
    let mut receipts_data = use_resource(move || {
        let principal_q = receipts_principal_q.clone();
        async move {
            let principal_id = if principal_q.is_empty() {
                None
            } else {
                Some(principal_q.as_str())
            };
            key_backup::list_recovery_receipts(None, principal_id).await
        }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("key_backup.title"),
                description: t("key_backup.subtitle"),
            }

            // ── Series filter ──
            div { class: "rounded-md border bg-card p-4",
                div { class: "grid gap-3 md:grid-cols-2",
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), "series_id" }
                        Input {
                            placeholder: "ck:backup_series:…".to_string(),
                            value: series_filter.read().clone(),
                            oninput: move |evt: FormEvent| series_filter.set(evt.value()),
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), "backup_class" }
                        Input {
                            placeholder: "e.g. did_recovery".to_string(),
                            value: backup_class_filter.read().clone(),
                            oninput: move |evt: FormEvent| backup_class_filter.set(evt.value()),
                        }
                    }
                }
                div { class: "mt-3",
                    Button {
                        size: ButtonSize::Sm,
                        onclick: move |_| series_data.restart(),
                        {t("key_backup.apply_filter")}
                    }
                }
            }

            div { class: "rounded-md border bg-card p-4",
                div { class: "space-y-1",
                    Label { class: "text-xs text-muted-foreground".to_string(), "principal_id" }
                    Input {
                        placeholder: t("key_backup.current_principal_when_empty"),
                        value: principal_filter.read().clone(),
                        oninput: move |evt: FormEvent| principal_filter.set(evt.value()),
                    }
                }
                div { class: "mt-3 flex gap-2",
                    Button {
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            policies_data.restart();
                            receipts_data.restart();
                        },
                        {t("key_backup.apply_recovery_filter")}
                    }
                }
            }

            // ── Series table ──
            div { class: "rounded-md border",
                div { class: "p-3 border-b font-medium text-sm", {t("key_backup.backup_series")} }
                match &*series_data.read() {
                    Some(Ok(resp)) => rsx! {
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "backup_id" }
                                    TableHead { "actor_id" }
                                    TableHead { "backup_class" }
                                    TableHead { "backup_version" }
                                    TableHead { "created_at" }
                                    TableHead { "ciphertext_digest" }
                                }
                            }
                            TableBody {
                                if resp.backups.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-6".to_string(),
                                            {t("key_backup.no_series")}
                                        }
                                    }
                                } else {
                                    for s in resp.backups.iter() {
                                        {render_series_row(s)}
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        ErrorBanner {
                            message: e.message.clone(),
                            on_retry: move |_| series_data.restart(),
                        }
                    },
                    None => rsx! { PageSkeleton {} },
                }
            }

            // ── Recovery policy history (read-only) ──
            div { class: "rounded-md border",
                div { class: "p-3 border-b font-medium text-sm",
                    {t("key_backup.recovery_policies")}
                }
                match &*policies_data.read() {
                    Some(Ok(resp)) => rsx! {
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "policy_id" }
                                    TableHead { "version" }
                                    TableHead { "trust_domain" }
                                    TableHead { "allowed_proof_kinds" }
                                    TableHead { "issued_at" }
                                    TableHead { "expires_at" }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-6".to_string(),
                                            {t("key_backup.no_policies")}
                                        }
                                    }
                                } else {
                                    for p in resp.data.iter() {
                                        {render_policy_row(p)}
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        ErrorBanner {
                            message: e.message.clone(),
                            on_retry: move |_| policies_data.restart(),
                        }
                    },
                    None => rsx! { PageSkeleton {} },
                }
            }

            // ── Recovery receipt history ──
            div { class: "rounded-md border",
                div { class: "p-3 border-b font-medium text-sm", {t("key_backup.recovery_receipts")} }
                match &*receipts_data.read() {
                    Some(Ok(resp)) => rsx! {
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "receipt_id" }
                                    TableHead { "recovery_session_id" }
                                    TableHead { "policy_id" }
                                    TableHead { "completed_at" }
                                    TableHead { "outcome" }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-6".to_string(),
                                            {t("key_backup.no_receipts")}
                                        }
                                    }
                                } else {
                                    for r in resp.data.iter() {
                                        TableRow {
                                            key: "{r.receipt_id}",
                                            TableCell { class: "font-mono text-xs".to_string(), "{r.receipt_id}" }
                                            TableCell { class: "font-mono text-xs".to_string(), "{r.recovery_session_id}" }
                                            TableCell { class: "font-mono text-xs".to_string(), "{r.policy_id}" }
                                            TableCell { class: "text-xs".to_string(),
                                                "{r.completed_at.clone().unwrap_or_else(|| \"-\".into())}"
                                            }
                                            TableCell {
                                                if r.outcome == "completed" {
                                                    Badge { variant: BadgeVariant::Default, "completed" }
                                                } else {
                                                    Badge { variant: BadgeVariant::Outline, "{r.outcome}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        ErrorBanner {
                            message: e.message.clone(),
                            on_retry: move |_| receipts_data.restart(),
                        }
                    },
                    None => rsx! { PageSkeleton {} },
                }
            }
        }
    }
}

fn render_series_row(s: &KeyBackupSummary) -> Element {
    let class = format!("{:?}", s.backup_class);
    let created_at = s.created_at.to_rfc3339();
    rsx! {
        TableRow {
            key: "{s.backup_id}",
            TableCell { class: "font-mono text-xs max-w-[180px] truncate".to_string(), "{s.backup_id}" }
            TableCell { class: "font-mono text-xs max-w-[180px] truncate".to_string(), "{s.actor_id}" }
            TableCell { "{class}" }
            TableCell { "{s.backup_version}" }
            TableCell { class: "text-xs".to_string(), "{created_at}" }
            TableCell { class: "font-mono text-xs max-w-[180px] truncate".to_string(), "{s.ciphertext_digest}" }
        }
    }
}

fn render_policy_row(p: &RecoveryPolicySummary) -> Element {
    let proof_kinds = if p.allowed_proof_kinds.is_empty() {
        // Explicit-revocation policy: an empty proof-kind set means
        // recovery is disabled under this policy.
        "(revoked)".to_string()
    } else {
        p.allowed_proof_kinds.join(", ")
    };
    let issued_at = p.issued_at.clone().unwrap_or_else(|| "-".into());
    let expires_at = p.expires_at.clone().unwrap_or_else(|| "-".into());
    rsx! {
        TableRow {
            key: "{p.policy_id}",
            TableCell { class: "font-mono text-xs".to_string(), "{p.policy_id}" }
            TableCell {
                Badge { variant: BadgeVariant::Secondary, "v{p.version}" }
            }
            TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{p.trust_domain}" }
            TableCell { class: "text-xs".to_string(), "{proof_kinds}" }
            TableCell { class: "text-xs".to_string(), "{issued_at}" }
            TableCell { class: "text-xs".to_string(), "{expires_at}" }
        }
    }
}
