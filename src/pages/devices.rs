use std::collections::HashSet;

use dioxus::prelude::*;

use crate::api::devices;
use crate::components::dangerous_action_dialog::DangerousActionDialog;
use crate::components::ui::auto_refresh::{
    self, AutoRefreshPicker, RefreshInterval,
};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::checkbox::{self, Checkbox};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::csv::{build_csv, export_to_csv};
use crate::utils::i18n::t;
use crate::utils::search::matches_name_or_id;

const PAGE_SIZE: u64 = 25;
const BULK_CONCURRENCY: usize = 5;
const AUTOREFRESH_STORAGE_KEY: &str = "sodmin.devices.autorefresh";

#[component]
pub fn DeviceList() -> Element {
    let mut search = use_signal(String::new);
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut show_delete_dialog = use_signal(|| None::<String>);
    let mut selected = use_signal::<HashSet<String>>(HashSet::new);
    let mut show_bulk_dialog = use_signal(|| false);
    let mut bulk_progress = use_signal::<Option<(usize, usize)>>(|| None);
    let mut autorefresh = use_signal(|| auto_refresh::load(AUTOREFRESH_STORAGE_KEY));

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let search_val = search.read().clone();

    let mut data = use_resource(move || {
        let s = search_val.clone();
        let cursor = cursor_snapshot.clone();
        async move { devices::list_devices(cursor.as_deref(), PAGE_SIZE, &s).await }
    });

    // Auto-refresh timer — re-installs whenever the chosen interval
    // changes. The previous handle is stored in a signal and
    // explicitly dropped (which cancels the underlying JS interval)
    // before a new one is created, so changing cadence doesn't leak
    // timers.
    let mut interval_handle = use_signal::<Option<gloo_timers::callback::Interval>>(|| None);
    use_effect(move || {
        // Read the signal INSIDE the effect closure so Dioxus tracks
        // the dependency and re-runs us whenever the picker changes.
        let choice = *autorefresh.read();
        // Drop any previous interval first — explicit so the timer
        // actually stops firing rather than just losing our handle.
        interval_handle.set(None);
        if let Some(ms) = choice.millis() {
            let mut data = data;
            let handle = gloo_timers::callback::Interval::new(ms, move || {
                data.restart();
            });
            interval_handle.set(Some(handle));
        }
    });

    // Snapshot the currently visible ids for "Select all" semantics.
    let visible_ids: Vec<String> = match &*data.read() {
        Some(Ok(resp)) => resp
            .data
            .iter()
            .filter(|d| {
                matches_name_or_id(
                    search.read().as_str(),
                    &d.id,
                    d.display_name.as_deref(),
                )
            })
            .map(|d| d.id.clone())
            .collect(),
        _ => Vec::new(),
    };
    let selected_on_page = visible_ids
        .iter()
        .filter(|id| selected.read().contains(id.as_str()))
        .count();
    let (header_checked, header_indeterminate) =
        checkbox::header_state(selected_on_page, visible_ids.len());
    let selected_count = selected.read().len();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("devices.title"),
                description: t("devices.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        if let Some(Ok(resp)) = data.read().as_ref() {
                            let rows: Vec<Vec<String>> = resp.data.iter().map(|d| vec![
                                d.id.clone(),
                                d.actor_id.clone().unwrap_or_default(),
                                d.display_name.clone().unwrap_or_default(),
                                d.device_type.clone().unwrap_or_default(),
                                d.verification_status.clone().unwrap_or_default(),
                                d.last_seen_ts.map(|t| t.to_string()).unwrap_or_default(),
                            ]).collect();
                            let csv = build_csv(
                                &["id", "actor_id", "display_name", "device_type", "verification_status", "last_seen_ts_ms"],
                                &rows,
                            );
                            export_to_csv("devices.csv", &csv);
                            show_toast("Devices CSV downloaded", ToastVariant::Success);
                        }
                    },
                    {t("common.export_csv")}
                }
            }

            div { class: "flex items-center gap-4 flex-wrap",
                div { class: "flex-1 min-w-[240px]",
                    SearchInput {
                        placeholder: t("devices.search"),
                        value: search.read().clone(),
                        oninput: move |evt: FormEvent| {
                            search.set(evt.value());
                            cursor_stack.set(vec![None::<String>]);
                        },
                    }
                }
                AutoRefreshPicker {
                    storage_key: AUTOREFRESH_STORAGE_KEY.to_string(),
                    value: *autorefresh.read(),
                    onchange: move |v: RefreshInterval| autorefresh.set(v),
                }
            }

            if selected_count > 0 {
                div { class: "flex items-center justify-between rounded-md border bg-accent/40 px-3 py-2",
                    div { class: "text-sm text-foreground",
                        "{selected_count} selected"
                    }
                    div { class: "flex items-center gap-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| selected.set(HashSet::new()),
                            "Clear"
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Sm,
                            onclick: move |_| show_bulk_dialog.set(true),
                            {format!("Bulk Revoke ({} selected)", selected_count)}
                        }
                    }
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => {
                    let next_cursor = resp.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let visible_ids = visible_ids.clone();
                    let visible_set: HashSet<String> = visible_ids.iter().cloned().collect();
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead {
                                            class: "w-10".to_string(),
                                            Checkbox {
                                                id: "devices-select-all".to_string(),
                                                aria_label: "Select all rows on this page".to_string(),
                                                checked: header_checked,
                                                indeterminate: header_indeterminate,
                                                onchange: move |_| {
                                                    let mut cur = selected.read().clone();
                                                    let all_selected = visible_set
                                                        .iter()
                                                        .all(|id| cur.contains(id));
                                                    if all_selected {
                                                        for id in visible_set.iter() {
                                                            cur.remove(id);
                                                        }
                                                    } else {
                                                        for id in visible_set.iter() {
                                                            cur.insert(id.clone());
                                                        }
                                                    }
                                                    selected.set(cur);
                                                },
                                            }
                                        }
                                        TableHead { {t("devices.id")} }
                                        TableHead { {t("devices.actor_id")} }
                                        TableHead { {t("devices.display_name")} }
                                        TableHead { {t("devices.device_type")} }
                                        TableHead { {t("devices.verification_status")} }
                                        TableHead { {t("devices.last_seen_ts")} }
                                        TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                    }
                                }
                                TableBody {
                                    if visible_ids.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("devices.no_devices")}
                                            }
                                        }
                                    } else {
                                        for device in resp.data.iter().filter(|d| matches_name_or_id(search.read().as_str(), &d.id, d.display_name.as_deref())) {
                                            {
                                                let id = device.id.clone();
                                                let actor_id = device.actor_id.clone().unwrap_or_else(|| "-".to_string());
                                                let display_name = device.display_name.clone().unwrap_or_else(|| "-".to_string());
                                                let device_type = device.device_type.clone().unwrap_or_else(|| "-".to_string());
                                                let verification = device.verification_status.clone().unwrap_or_else(|| "-".to_string());
                                                let last_seen = device.last_seen_ts.map_or_else(|| "-".to_string(), |ts| {
                                                    let secs = (ts / 1000) as i64;
                                                    chrono::DateTime::from_timestamp(secs, 0)
                                                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                                        .unwrap_or_else(|| "-".to_string())
                                                });

                                                let id_for_delete = id.clone();
                                                let id_for_check = id.clone();
                                                let is_checked = selected.read().contains(&id);
                                                let check_id = format!("dev-check-{}", id);

                                                rsx! {
                                                    TableRow {
                                                        TableCell {
                                                            class: "w-10".to_string(),
                                                            Checkbox {
                                                                id: check_id,
                                                                aria_label: format!("Select device {}", id),
                                                                checked: is_checked,
                                                                onchange: move |_| {
                                                                    let mut cur = selected.read().clone();
                                                                    if cur.contains(&id_for_check) {
                                                                        cur.remove(&id_for_check);
                                                                    } else {
                                                                        cur.insert(id_for_check.clone());
                                                                    }
                                                                    selected.set(cur);
                                                                },
                                                            }
                                                        }
                                                        TableCell { class: "font-medium".to_string(), "{id}" }
                                                        TableCell { class: "max-w-[200px] truncate".to_string(), "{actor_id}" }
                                                        TableCell { "{display_name}" }
                                                        TableCell { "{device_type}" }
                                                        TableCell { "{verification}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{last_seen}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                onclick: {
                                                                    let id = id_for_delete.clone();
                                                                    move |_| show_delete_dialog.set(Some(id.clone()))
                                                                },
                                                                {t("common.delete")}
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

                        CursorPagination {
                            depth: stack_depth,
                            has_next: next_cursor.is_some(),
                            on_prev: move |_| {
                                let mut new_stack = cursor_stack.read().clone();
                                if new_stack.len() > 1 {
                                    new_stack.pop();
                                    cursor_stack.set(new_stack);
                                }
                            },
                            on_next: move |_| {
                                if let Some(c) = next_cursor.clone() {
                                    let mut new_stack = cursor_stack.read().clone();
                                    new_stack.push(Some(c));
                                    cursor_stack.set(new_stack);
                                }
                            },
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|b| b.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }

            if let Some((done, total)) = *bulk_progress.read() {
                div { class: "rounded-md border bg-muted/40 px-3 py-2 text-sm",
                    "Bulk revoke in progress: {done} / {total}"
                }
            }
        }

        // Single-row delete confirmation.
        {
            let pending = show_delete_dialog.read().clone();
            let phrase = pending
                .as_deref()
                .map(|id| crate::components::dangerous_action_dialog::device_revoke_phrase(id, 4))
                .unwrap_or_default();
            let pending_id_desc = pending.clone().unwrap_or_default();
            rsx! {
                DangerousActionDialog {
                    open: pending.is_some(),
                    title: t("common.delete"),
                    description: format!(
                        "Revoke device {}? This signs out the session, drops device keys, and cannot be undone.",
                        pending_id_desc
                    ),
                    confirmation_phrase: phrase,
                    confirm_text: t("common.delete"),
                    cancel_text: t("common.cancel"),
                    on_confirm: move |_| {
                        if let Some(id) = show_delete_dialog.read().clone() {
                            let id = id.clone();
                            spawn(async move {
                                match devices::delete_device(&id).await {
                                    Ok(_) => {
                                        show_toast("Device deleted", ToastVariant::Success);
                                        data.restart();
                                    }
                                    Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                }
                            });
                        }
                        show_delete_dialog.set(None);
                    },
                    on_cancel: move |_| show_delete_dialog.set(None),
                }
            }
        }

        // Bulk-revoke confirmation. Uses the literal phrase `REVOKE` to
        // gate the destructive button — distinct ids per row don't
        // generalise to a multi-row gate.
        {
            let open = *show_bulk_dialog.read();
            let count = selected_count;
            rsx! {
                DangerousActionDialog {
                    open,
                    title: format!("Bulk revoke {} devices?", count),
                    description: "All selected devices will be signed out and their keys dropped. Each failure is reported individually; successful revokes are not rolled back.".to_string(),
                    confirmation_phrase: "REVOKE".to_string(),
                    confirm_text: "Bulk Revoke".to_string(),
                    cancel_text: t("common.cancel"),
                    on_cancel: move |_| show_bulk_dialog.set(false),
                    on_confirm: move |_| {
                        let ids: Vec<String> = selected.read().iter().cloned().collect();
                        show_bulk_dialog.set(false);
                        if ids.is_empty() {
                            return;
                        }
                        bulk_progress.set(Some((0, ids.len())));
                        spawn(async move {
                            let total = ids.len();
                            let (ok, failed) = run_bulk_revoke(ids, BULK_CONCURRENCY, move |done| {
                                bulk_progress.set(Some((done, total)));
                            })
                            .await;
                            bulk_progress.set(None);
                            selected.set(HashSet::new());
                            data.restart();
                            if !failed.is_empty() {
                                let preview: Vec<String> = failed
                                    .iter()
                                    .take(3)
                                    .map(|(id, msg)| format!("{}: {}", id, msg))
                                    .collect();
                                let suffix = if failed.len() > 3 {
                                    format!(" (+{} more)", failed.len() - 3)
                                } else {
                                    String::new()
                                };
                                show_toast(
                                    &format!(
                                        "Revoked {} / {}. Failed: {}{}",
                                        ok,
                                        total,
                                        preview.join("; "),
                                        suffix
                                    ),
                                    ToastVariant::Error,
                                );
                            } else {
                                show_toast(
                                    &format!("Revoked {} device(s)", ok),
                                    ToastVariant::Success,
                                );
                            }
                        });
                    },
                }
            }
        }
    }
}

/// Bounded-concurrency bulk revoke. Walks `ids` in chunks of `concurrency`
/// (sequential `join_all` chunks rather than a real channel — `wasm32`
/// is single-threaded so the chunk pattern keeps the in-flight count at
/// or below the cap without a buffered semaphore). Single-row failures
/// don't abort the rest; the caller surfaces them via toast.
async fn run_bulk_revoke<F>(
    ids: Vec<String>,
    concurrency: usize,
    mut on_progress: F,
) -> (usize, Vec<(String, String)>)
where
    F: FnMut(usize) + 'static,
{
    let mut ok = 0usize;
    let mut failed: Vec<(String, String)> = Vec::new();
    let mut done = 0usize;
    let chunk = concurrency.max(1);
    for batch in ids.chunks(chunk) {
        let futures: Vec<_> = batch
            .iter()
            .cloned()
            .map(|id| async move {
                let res = devices::revoke_device(&id).await;
                (id, res)
            })
            .collect();
        let results = futures_join_all(futures).await;
        for (id, res) in results {
            done += 1;
            match res {
                Ok(_) => ok += 1,
                Err(e) => failed.push((id, e.message.clone())),
            }
        }
        on_progress(done);
    }
    (ok, failed)
}

// Concurrent `join_all` shim — exposed as `pub` so the agents admin
// bulk revoke helper can reuse the same primitive without taking a
// `futures` crate dep just for one combinator.
pub use crate::utils::futures::join_all as futures_join_all;
