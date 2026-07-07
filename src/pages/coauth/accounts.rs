use dioxus::prelude::*;

use crate::api::coauth::{self, AccountListFilter};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::Input;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::router::Route;

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
    let mut handle_filter = use_signal(String::new);
    let mut display_name_filter = use_signal(String::new);

    let mut data = use_resource(move || {
        let cursor = cursor_stack.read().last().cloned().unwrap_or(None);
        let search = search.read().clone();
        let handle = handle_filter.read().clone();
        let display_name = display_name_filter.read().clone();
        async move {
            let filter = AccountListFilter {
                handle,
                display_name,
            };
            coauth::list_accounts_cursor(cursor.as_deref(), PAGE_SIZE, &search, &filter).await
        }
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
                title: "coauth Accounts".to_string(),
                description: "Cursor-paginated account view backed by /accounts. Use the filter inputs to narrow by free text, handle, or display name; the server's links.next is the only way to advance.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    "Refresh"
                }
            }

            div { class: "rounded-lg border p-4 space-y-3",
                div { class: "text-sm text-muted-foreground",
                    "Filters apply server-side via filter[search], filter[handle], filter[display_name]. Pagination uses the JSON:API cursor model (cursor=base64url, limit=N)."
                }
                div { class: "grid gap-3 md:grid-cols-3",
                    Input {
                        value: search.read().clone(),
                        placeholder: "Search (any field)".to_string(),
                        oninput: move |evt: FormEvent| {
                            reset_to_first_page();
                            search.set(evt.value());
                        },
                    }
                    Input {
                        value: handle_filter.read().clone(),
                        placeholder: "Filter by handle".to_string(),
                        oninput: move |evt: FormEvent| {
                            reset_to_first_page();
                            handle_filter.set(evt.value());
                        },
                    }
                    Input {
                        value: display_name_filter.read().clone(),
                        placeholder: "Filter by display name".to_string(),
                        oninput: move |evt: FormEvent| {
                            reset_to_first_page();
                            display_name_filter.set(evt.value());
                        },
                    }
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let count_label = match page.total {
                        Some(n) => format!("{} matching accounts on the server", n),
                        None => "server did not include a total count".to_string(),
                    };
                    let next_cursor = page.next_cursor.clone();
                    let row_count = page.data.len();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        div { class: "rounded-lg border p-4 text-sm text-muted-foreground",
                            "Loaded "
                            span { class: "font-medium text-foreground", "{row_count}" }
                            " account records this page. "
                            span { class: "font-medium text-foreground", "{count_label}" }
                            "."
                        }
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { "Account ID" }
                                        TableHead { "Handle" }
                                        TableHead { "Display Name" }
                                        TableHead { "Status" }
                                        TableHead { "Primary DID" }
                                        TableHead { "Bridge" }
                                        TableHead { class: "text-right".to_string(), "Actions" }
                                    }
                                }
                                TableBody {
                                    if page.data.is_empty() {
                                        TableRow {
                                            TableCell { class: "py-8 text-center text-muted-foreground".to_string(), colspan: 99,
                                                "No accounts match the current filter."
                                            }
                                        }
                                    } else {
                                        for account in page.data.iter() {
                                            {
                                                let account_id = account.id.clone();
                                                let handle = account.username.clone().unwrap_or_else(|| "-".to_string());
                                                let display_name = account.display_name.clone().unwrap_or_else(|| "-".to_string());
                                                let primary_did = account.primary_did.clone().unwrap_or_else(|| "-".to_string());
                                                let bridge_status = account.bridge_status.clone();
                                                let status = account.lifecycle_label();

                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-medium".to_string(), "{account_id}" }
                                                        TableCell { "{handle}" }
                                                        TableCell { "{display_name}" }
                                                        TableCell { "{status}" }
                                                        TableCell { "{primary_did}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{bridge_status}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            Link {
                                                                to: Route::CoauthAccountShow { account_id: account_id.clone() },
                                                                class: "inline-flex h-9 items-center rounded-md border px-3 text-sm font-medium transition-colors hover:bg-accent".to_string(),
                                                                "Open"
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
                                "Page "
                                span { class: "font-medium text-foreground", "{stack_depth}" }
                                if next_cursor.is_some() {
                                    " (more available)"
                                } else {
                                    " (last page)"
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
                                    "Previous"
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
                                    "Next"
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
