//! B-C key-backup admin page (`/key-backup`).
//!
//! Three panes:
//!   1. Backup series list with frontier status + per-series 3-class 409 counters
//!      (`series_chain_broken` / `series_seq_not_monotonic` / `series_predecessor_not_found`).
//!   2. Recovery policy editor (lifecycle pending/active/retired; KDF profile; epoch hash). Deep
//!      validators are `TODO(P3-impl)` server-side.
//!   3. Recovery receipt history.

use dioxus::prelude::*;

use crate::api::key_backup;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::RecoveryPolicy;

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
                title: "Key backup".to_string(),
                description: "Self key backups plus recovery policies and receipts filtered by principal_id when authorized.".to_string(),
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
                        "Apply filter"
                    }
                }
            }

            div { class: "rounded-md border bg-card p-4",
                div { class: "space-y-1",
                    Label { class: "text-xs text-muted-foreground".to_string(), "principal_id" }
                    Input {
                        placeholder: "current principal when empty".to_string(),
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
                        "Apply recovery filter"
                    }
                }
            }

            // ── Series table ──
            div { class: "rounded-md border",
                div { class: "p-3 border-b font-medium text-sm", "Backup series" }
                match &*series_data.read() {
                    Some(Ok(resp)) => rsx! {
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "series_id" }
                                    TableHead { "backup_class" }
                                    TableHead { "frontier_seq" }
                                    TableHead { "frontier_ref" }
                                    TableHead { "series_seq" }
                                    TableHead { "chain_broken" }
                                    TableHead { "not_monotonic" }
                                    TableHead { "predecessor_not_found" }
                                }
                            }
                            TableBody {
                                if resp.backups.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-6".to_string(),
                                            "No backup series found."
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

            // ── Recovery policy editor ──
            div { class: "rounded-md border",
                div { class: "p-3 border-b font-medium text-sm",
                    "Recovery policies"
                }
                match &*policies_data.read() {
                    Some(Ok(resp)) => rsx! {
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "policy_id" }
                                    TableHead { "lifecycle" }
                                    TableHead { "kdf_profile" }
                                    TableHead { "epoch_hash" }
                                    TableHead { "actions" }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-6".to_string(),
                                            "No recovery policies."
                                        }
                                    }
                                } else {
                                    for p in resp.data.iter() {
                                        {render_policy_row(p, move |_| policies_data.restart())}
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
                div { class: "p-3 border-b font-medium text-sm", "Recovery receipts" }
                match &*receipts_data.read() {
                    Some(Ok(resp)) => rsx! {
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "receipt_id" }
                                    TableHead { "session_id" }
                                    TableHead { "policy_id" }
                                    TableHead { "issued_at" }
                                    TableHead { "verified" }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-6".to_string(),
                                            "No receipts yet."
                                        }
                                    }
                                } else {
                                    for r in resp.data.iter() {
                                        TableRow {
                                            TableCell { class: "font-mono text-xs".to_string(), "{r.receipt_id}" }
                                            TableCell { class: "font-mono text-xs".to_string(),
                                                "{r.session_id.clone().unwrap_or_else(|| \"-\".into())}"
                                            }
                                            TableCell { class: "font-mono text-xs".to_string(),
                                                "{r.policy_id.clone().unwrap_or_else(|| \"-\".into())}"
                                            }
                                            TableCell { class: "text-xs".to_string(),
                                                "{r.issued_at.clone().unwrap_or_else(|| \"-\".into())}"
                                            }
                                            TableCell {
                                                if r.verified {
                                                    Badge { variant: BadgeVariant::Default, "verified" }
                                                } else {
                                                    Badge { variant: BadgeVariant::Outline, "unverified" }
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

fn render_series_row(s: &crate::types::KeyBackupSeries) -> Element {
    let class = s.backup_class.clone().unwrap_or_else(|| "-".into());
    let frontier_ref = s.frontier_ref.clone().unwrap_or_else(|| "-".into());
    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(), "{s.series_id}" }
            TableCell { "{class}" }
            TableCell { "{s.frontier_seq}" }
            TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{frontier_ref}" }
            TableCell { "{s.series_seq}" }
            TableCell { class: "text-destructive".to_string(), "{s.series_chain_broken_count}" }
            TableCell { class: "text-destructive".to_string(), "{s.series_seq_not_monotonic_count}" }
            TableCell { class: "text-destructive".to_string(), "{s.series_predecessor_not_found_count}" }
        }
    }
}

fn render_policy_row(p: &RecoveryPolicy, on_change: impl FnMut(()) + 'static) -> Element {
    let mut on_change = on_change;
    let policy = p.clone();
    let lifecycle = p.lifecycle.clone();
    let kdf = p.kdf_profile.clone().unwrap_or_default();
    let epoch = p.epoch_hash.clone().unwrap_or_default();

    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(), "{policy.policy_id}" }
            TableCell {
                Badge {
                    variant: match lifecycle.as_str() {
                        "active" => BadgeVariant::Default,
                        "pending" => BadgeVariant::Outline,
                        "retired" => BadgeVariant::Secondary,
                        _ => BadgeVariant::Outline,
                    },
                    "{lifecycle}"
                }
            }
            TableCell { class: "font-mono text-xs".to_string(), "{kdf}" }
            TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{epoch}" }
            TableCell {
                Button {
                    size: ButtonSize::Sm,
                    variant: ButtonVariant::Outline,
                    // TODO(P3-impl): inline-edit form with lifecycle
                    // dropdown + epoch-hash validator. For now the
                    // button just round-trips the existing record so
                    // the PUT wire path is exercised.
                    onclick: move |_| {
                        let p = policy.clone();
                        spawn(async move {
                            match key_backup::upsert_recovery_policy(&p).await {
                                Ok(_) => show_toast("Policy saved", ToastVariant::Success),
                                Err(e) => show_toast(&e.message, ToastVariant::Error),
                            }
                        });
                        on_change(());
                    },
                    "Save"
                }
            }
        }
    }
}
