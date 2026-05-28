use dioxus::prelude::*;

use crate::api::federation;
use crate::components::ui::auto_refresh::{self, AutoRefreshPicker, RefreshInterval};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label, SearchInput};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::i18n::t;
use crate::utils::search::matches_name_or_id;

const PAGE_SIZE: u64 = 25;
const AUTOREFRESH_STORAGE_KEY: &str = "sodmin.federation.autorefresh";

#[component]
pub fn FederationList() -> Element {
    let mut peer_cursors = use_signal(|| vec![None::<String>]);
    let mut rule_cursors = use_signal(|| vec![None::<String>]);
    let mut search = use_signal(String::new);
    let mut show_add_rule = use_signal(|| false);
    let mut show_delete_rule = use_signal(|| None::<String>);
    let mut new_rule_domain = use_signal(String::new);
    let mut new_rule_polarity = use_signal(|| "allow".to_string());
    let mut new_rule_action = use_signal(|| "allow_federation".to_string());
    let mut new_rule_allowlist_enforced = use_signal(|| false);
    let mut add_loading = use_signal(|| false);
    let mut autorefresh = use_signal(|| auto_refresh::load(AUTOREFRESH_STORAGE_KEY));

    let peer_cursor = peer_cursors.read().last().cloned().unwrap_or(None);
    let rule_cursor = rule_cursors.read().last().cloned().unwrap_or(None);
    let search_val = search.read().clone();

    let mut peers = use_resource(move || {
        let c = peer_cursor.clone();
        let s = search_val.clone();
        async move { federation::list_federation_peers(c.as_deref(), PAGE_SIZE, &s).await }
    });

    let mut rules = use_resource(move || {
        let c = rule_cursor.clone();
        async move { federation::list_federation_allow_rules(c.as_deref(), PAGE_SIZE).await }
    });

    let mut interval_handle = use_signal::<Option<gloo_timers::callback::Interval>>(|| None);
    use_effect(move || {
        let choice = *autorefresh.read();
        interval_handle.set(None);
        if let Some(ms) = choice.millis() {
            let mut peers = peers;
            let mut rules = rules;
            let handle = gloo_timers::callback::Interval::new(ms, move || {
                peers.restart();
                rules.restart();
            });
            interval_handle.set(Some(handle));
        }
    });

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: t("federation.title"),
                description: t("federation.subtitle"),
            }

            div { class: "flex items-center gap-4 flex-wrap",
                div { class: "flex-1 min-w-[240px]",
                    SearchInput {
                        placeholder: t("federation.domain"),
                        value: search.read().clone(),
                        oninput: move |evt: FormEvent| {
                            search.set(evt.value());
                            peer_cursors.set(vec![None::<String>]);
                        },
                    }
                }
                AutoRefreshPicker {
                    storage_key: AUTOREFRESH_STORAGE_KEY.to_string(),
                    value: *autorefresh.read(),
                    onchange: move |v: RefreshInterval| autorefresh.set(v),
                }
            }

            Card {
                CardHeader {
                    CardTitle { {t("federation.peers")} }
                }
                CardContent {
                    match &*peers.read() {
                        Some(Ok(data)) => {
                            let next_cursor = data.next_cursor.clone();
                            let depth = peer_cursors.read().len();
                            let search_for_filter = search.read().clone();
                            rsx! {
                                div { class: "rounded-md border",
                                    Table {
                                        TableHeader {
                                            TableRow {
                                                TableHead { {t("federation.domain")} }
                                                TableHead { {t("federation.status")} }
                                                TableHead { {t("federation.trust_level")} }
                                                TableHead { {t("federation.last_successful_txn")} }
                                                TableHead { {t("federation.last_error")} }
                                            }
                                        }
                                        TableBody {
                                            if data.data.is_empty() {
                                                TableRow {
                                                    TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                        {t("federation.no_peers")}
                                                    }
                                                }
                                            } else {
                                                for peer in data.data.iter().filter(|p| matches_name_or_id(&search_for_filter, &p.domain, None)) {
                                                    {
                                                        let domain = peer.domain.clone();
                                                        let status = peer.status.clone().unwrap_or_else(|| "-".to_string());
                                                        let trust_level = peer.trust_level.clone().unwrap_or_else(|| "-".to_string());
                                                        let last_txn = peer.last_successful_txn.clone().unwrap_or_else(|| "-".to_string());
                                                        let last_error = peer.last_error.clone().unwrap_or_else(|| "-".to_string());

                                                        rsx! {
                                                            TableRow {
                                                                TableCell {
                                                                    Link {
                                                                        to: Route::FederationShow { domain: urlencoding::encode(&domain).to_string() },
                                                                        class: "font-medium text-primary hover:underline",
                                                                        "{domain}"
                                                                    }
                                                                }
                                                                TableCell {
                                                                    Badge { variant: BadgeVariant::Secondary, "{status}" }
                                                                }
                                                                TableCell { "{trust_level}" }
                                                                TableCell { class: "text-muted-foreground".to_string(), "{last_txn}" }
                                                                TableCell { class: "max-w-[200px] truncate text-destructive".to_string(), "{last_error}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                CursorPagination {
                                    depth,
                                    has_next: next_cursor.is_some(),
                                    on_prev: move |_| {
                                        let mut new_stack = peer_cursors.read().clone();
                                        if new_stack.len() > 1 {
                                            new_stack.pop();
                                            peer_cursors.set(new_stack);
                                        }
                                    },
                                    on_next: move |_| {
                                        if let Some(c) = next_cursor.clone() {
                                            let mut new_stack = peer_cursors.read().clone();
                                            new_stack.push(Some(c));
                                            peer_cursors.set(new_stack);
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
                                on_retry: move |_| peers.restart(),
                            }
                        },
                        None => rsx! { PageSkeleton {} },
                    }
                }
            }

            Card {
                CardHeader {
                    div { class: "flex items-center justify-between",
                        CardTitle { {t("federation.allow_rules")} }
                        Button {
                            variant: ButtonVariant::Default,
                            onclick: move |_| show_add_rule.set(true),
                            {t("federation.add_rule")}
                        }
                    }
                }
                CardContent {
                    match &*rules.read() {
                        Some(Ok(data)) => {
                            let next_cursor = data.next_cursor.clone();
                            let depth = rule_cursors.read().len();
                            rsx! {
                                div { class: "rounded-md border",
                                    Table {
                                        TableHeader {
                                            TableRow {
                                                TableHead { {t("federation.id")} }
                                                TableHead { {t("federation.domain")} }
                                                TableHead { {t("federation.rule_type")} }
                                                TableHead { "Polarity" }
                                                TableHead { "Action" }
                                                TableHead { "Allowlist" }
                                                TableHead { {t("federation.created_at")} }
                                                TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                            }
                                        }
                                        TableBody {
                                            if data.data.is_empty() {
                                                TableRow {
                                                    TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                        {t("federation.no_rules")}
                                                    }
                                                }
                                            } else {
                                                for rule in data.data.iter() {
                                                    {
                                                        let id = rule.id.clone();
                                                        let domain = rule.domain.clone();
                                                        let rule_type = rule.rule_type.clone().unwrap_or_else(|| "-".to_string());
                                                        let polarity = rule.polarity.clone().unwrap_or_else(|| rule_type.clone());
                                                        let action = rule.action.clone().unwrap_or_else(|| "-".to_string());
                                                        let allowlist = if rule.allowlist_enforced.unwrap_or(false) { "enforced" } else { "-" };
                                                        let created_at = rule.created_at.clone().unwrap_or_else(|| "-".to_string());
                                                        let id_for_delete = id.clone();

                                                        rsx! {
                                                            TableRow {
                                                                TableCell { class: "font-medium".to_string(), "{id}" }
                                                                TableCell { "{domain}" }
                                                                TableCell { "{rule_type}" }
                                                                TableCell { "{polarity}" }
                                                                TableCell { "{action}" }
                                                                TableCell { "{allowlist}" }
                                                                TableCell { class: "text-muted-foreground".to_string(), "{created_at}" }
                                                                TableCell { class: "text-right".to_string(),
                                                                    Button {
                                                                        variant: ButtonVariant::Ghost,
                                                                        onclick: {
                                                                            let id = id_for_delete.clone();
                                                                            move |_| show_delete_rule.set(Some(id.clone()))
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
                                    depth,
                                    has_next: next_cursor.is_some(),
                                    on_prev: move |_| {
                                        let mut new_stack = rule_cursors.read().clone();
                                        if new_stack.len() > 1 {
                                            new_stack.pop();
                                            rule_cursors.set(new_stack);
                                        }
                                    },
                                    on_next: move |_| {
                                        if let Some(c) = next_cursor.clone() {
                                            let mut new_stack = rule_cursors.read().clone();
                                            new_stack.push(Some(c));
                                            rule_cursors.set(new_stack);
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
                                on_retry: move |_| rules.restart(),
                            }
                        },
                        None => rsx! { PageSkeleton {} },
                    }
                }
            }
        }

        if *show_add_rule.read() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center",
                    div { class: "fixed inset-0 bg-black/80", onclick: move |_| show_add_rule.set(false) }
                    div { class: "relative z-50 w-full max-w-md rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                        h2 { class: "text-lg font-semibold", {t("federation.add_rule")} }
                        div { class: "space-y-1",
                            Label { r#for: "rule-domain".to_string(), {t("federation.domain")} }
                            Input {
                                id: "rule-domain".to_string(),
                                value: new_rule_domain.read().clone(),
                                oninput: move |evt: FormEvent| new_rule_domain.set(evt.value()),
                            }
                        }
                        div { class: "grid gap-4 sm:grid-cols-2",
                            div { class: "space-y-1",
                                Label { r#for: "rule-polarity".to_string(), "Polarity" }
                                select {
                                    id: "rule-polarity",
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                    value: new_rule_polarity.read().clone(),
                                    onchange: move |evt| {
                                        let value = evt.value();
                                        new_rule_polarity.set(value.clone());
                                        new_rule_action.set(match value.as_str() {
                                            "deny" => "deny_federation".to_string(),
                                            "block" => "block_federation".to_string(),
                                            _ => "allow_federation".to_string(),
                                        });
                                    },
                                    option { value: "allow", "allow" }
                                    option { value: "deny", "deny" }
                                    option { value: "block", "block" }
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "rule-action".to_string(), "Action" }
                                select {
                                    id: "rule-action",
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                    value: new_rule_action.read().clone(),
                                    onchange: move |evt| new_rule_action.set(evt.value()),
                                    option { value: "allow_federation", "allow_federation" }
                                    option { value: "deny_federation", "deny_federation" }
                                    option { value: "block_federation", "block_federation" }
                                    option { value: "defederate", "defederate" }
                                }
                            }
                        }
                        label { class: "flex items-center gap-2 text-sm",
                            Checkbox {
                                id: "rule-allowlist-enforced".to_string(),
                                checked: *new_rule_allowlist_enforced.read(),
                                aria_label: "Enforce allowlist".to_string(),
                                onchange: move |checked| new_rule_allowlist_enforced.set(checked),
                            }
                            span { "Enforce allowlist mode" }
                        }
                        div { class: "flex justify-end gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| show_add_rule.set(false),
                                {t("common.cancel")}
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *add_loading.read(),
                                onclick: move |_| {
                                    add_loading.set(true);
                                    let domain = new_rule_domain.read().clone();
                                    let polarity = new_rule_polarity.read().clone();
                                    let action = new_rule_action.read().clone();
                                    let allowlist_enforced = *new_rule_allowlist_enforced.read();
                                    spawn(async move {
                                        let request = crate::types::AddFederationRuleRequest {
                                            domain,
                                            rule_type: Some(polarity.clone()),
                                            polarity: Some(polarity),
                                            action: Some(action),
                                            allowlist_enforced: Some(allowlist_enforced),
                                        };
                                        match federation::add_federation_rule(&request).await {
                                            Ok(_) => {
                                                show_toast("Rule added", ToastVariant::Success);
                                                show_add_rule.set(false);
                                                new_rule_domain.set(String::new());
                                                new_rule_polarity.set("allow".to_string());
                                                new_rule_action.set("allow_federation".to_string());
                                                new_rule_allowlist_enforced.set(false);
                                                rules.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                                        add_loading.set(false);
                                    });
                                },
                                {t("common.create")}
                            }
                        }
                    }
                }
            }

        ConfirmDialog {
            open: show_delete_rule.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this allow rule?".to_string(),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_delete_rule.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match federation::delete_federation_allow_rule(&id).await {
                            Ok(_) => {
                                show_toast("Rule deleted", ToastVariant::Success);
                                rules.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete_rule.set(None);
            },
            on_cancel: move |_| show_delete_rule.set(None),
        }
    }
}
