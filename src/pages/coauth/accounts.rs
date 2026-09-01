use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::Input;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

/// Cursor stack entry — `None` means "first page", `Some(c)` means "the
/// cursor that fetched this page". Pushing a new cursor when the user
/// clicks "Next" lets us implement client-side back navigation without
/// a `prev` link from the server.
#[component]
pub fn AccountsPage() -> Element {
    // Forward navigation stack. `cursor_stack[stack_idx]` is the cursor
    // used to load the page currently on screen; `None` is the first
    // page. Clicking "Next" pushes the server's `next_cursor` onto the
    // stack; clicking "Previous" pops one off.
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut search = use_signal(String::new);

    let mut data = use_resource(move || {
        let cursor = cursor_stack.read().last().cloned().unwrap_or(None);
        let search = search.read().clone();
        async move { coauth::list_accounts_cursor(cursor.as_deref(), PAGE_SIZE, &search).await }
    });

    // Reset back to the first page when any filter input changes. The
    // closure that runs on input is responsible for resetting the cursor
    // stack so the resource snapshot below picks up the change.
    let mut reset_to_first_page = move || {
        cursor_stack.set(vec![None]);
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth_accounts.title"),
                description: t("coauth_accounts.description"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "rounded-lg border p-4 space-y-3",
                div { class: "text-sm text-muted-foreground",
                    {t("coauth_accounts.filters_hint")}
                }
                div { class: "max-w-md",
                    Input {
                        value: search.read().clone(),
                        placeholder: t("coauth_accounts.search_placeholder"),
                        oninput: move |evt: FormEvent| {
                            reset_to_first_page();
                            search.set(evt.value());
                        },
                    }
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let count_label = match page.total {
                        Some(n) => t("coauth_accounts.count_matching").replace("{count}", &n.to_string()),
                        None => t("coauth_accounts.count_unknown"),
                    };
                    let next_cursor = page.next_cursor.clone();
                    let row_count = page.data.len();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        div { class: "rounded-lg border p-4 text-sm text-muted-foreground",
                            {t("coauth_accounts.loaded_prefix")}
                            span { class: "font-medium text-foreground", "{row_count}" }
                            {t("coauth_accounts.loaded_suffix")}
                            span { class: "font-medium text-foreground", "{count_label}" }
                            {t("coauth_accounts.sentence_end")}
                        }
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("coauth_account_detail.account_id")} }
                                        TableHead { {t("coauth_account_detail.handle")} }
                                        TableHead { {t("coauth_accounts.display_name")} }
                                        TableHead { {t("common.status")} }
                                        TableHead { {t("coauth_account_detail.primary_principal_id")} }
                                        TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                    }
                                }
                                TableBody {
                                    if page.data.is_empty() {
                                        TableRow {
                                            TableCell { class: "py-8 text-center text-muted-foreground".to_string(), colspan: 99,
                                                {t("coauth_accounts.empty")}
                                            }
                                        }
                                    } else {
                                        for account in page.data.iter() {
                                            {
                                                let account_id = account.id.clone();
                                                let handle = account.username.clone().unwrap_or_else(|| "-".to_string());
                                                let display_name = account.display_name.clone().unwrap_or_else(|| "-".to_string());
                                                let primary_principal_id = account
                                                    .primary_principal_id
                                                    .as_ref()
                                                    .map_or("-", arkret_identifiers::DidCoreId::as_str);
                                                let status = account.lifecycle_label();

                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-medium".to_string(), "{account_id}" }
                                                        TableCell { "{handle}" }
                                                        TableCell { "{display_name}" }
                                                        TableCell { "{status}" }
                                                        TableCell { "{primary_principal_id}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            Link {
                                                                to: Route::CoauthAccountShow { account_id: account_id.clone() },
                                                                class: "inline-flex h-9 items-center rounded-md border px-3 text-sm font-medium transition-colors hover:bg-accent".to_string(),
                                                                {t("common.open")}
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

                        div { class: "flex items-center justify-between px-2 py-4",
                            div { class: "text-sm text-muted-foreground",
                                {t("coauth_accounts.page_prefix")}
                                span { class: "font-medium text-foreground", "{stack_depth}" }
                                if next_cursor.is_some() {
                                    {t("coauth_accounts.page_more")}
                                } else {
                                    {t("coauth_accounts.page_last")}
                                }
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
                },
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
