use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::Input;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::router::Route;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AccountsPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut search = use_signal(String::new);
    let page_val = *page.read();

    let mut data = use_resource(move || {
        let search_val = search.read().clone();
        async move { coauth::list_accounts(page_val, PAGE_SIZE, &search_val).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "coauth Accounts".to_string(),
                description: "Account-first view over the current coauth admin account contract. DID bindings now come from the dedicated account surface; claim and grant inventory still trails behind.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    "Refresh"
                }
            }

            div { class: "rounded-lg border p-4 space-y-3",
                div { class: "text-sm text-muted-foreground",
                    "This page now reads coauth account summaries from /accounts. TODO(contract): replace the temporary page-number shim with cursor-native pagination once sodmin adopts the admin cursor model."
                }
                Input {
                    value: search.read().clone(),
                    placeholder: "Search by account id, handle, display name, or email".to_string(),
                    oninput: move |evt: FormEvent| {
                        page.set(1);
                        search.set(evt.value());
                    },
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-lg border p-4 text-sm text-muted-foreground",
                        "Loaded "
                        span { class: "font-medium text-foreground", "{resp.data.len()}" }
                        " account records from the current coauth admin surface. Total available: "
                        span { class: "font-medium text-foreground", "{resp.total}" }
                        ". TODO(contract): switch to generated OpenAPI DTOs once the coauth account contract is frozen."
                    }
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "Account ID" }
                                    TableHead { "Handle" }
                                    TableHead { "Display Name" }
                                    TableHead { "Email" }
                                    TableHead { "Status" }
                                    TableHead { "Primary DID" }
                                    TableHead { "Bridge" }
                                    TableHead { class: "text-right".to_string(), "Actions" }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "py-8 text-center text-muted-foreground".to_string(), colspan: 99,
                                            "No accounts found."
                                        }
                                    }
                                } else {
                                    for account in resp.data.iter() {
                                        {
                                            let account_id = account.id.clone();
                                            let handle = account.username.clone().unwrap_or_else(|| "-".to_string());
                                            let display_name = account.display_name.clone().unwrap_or_else(|| "-".to_string());
                                            let email = account.email.clone().unwrap_or_else(|| "-".to_string());
                                            let primary_did = account.primary_did.clone().unwrap_or_else(|| "-".to_string());
                                            let bridge_status = account.bridge_status.clone();
                                            let status = if account.is_deactivated {
                                                "Deactivated"
                                            } else if account.is_locked {
                                                "Locked"
                                            } else {
                                                "Active"
                                            };

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{account_id}" }
                                                    TableCell { "{handle}" }
                                                    TableCell { "{display_name}" }
                                                    TableCell { "{email}" }
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

                    Pagination {
                        page: page_val,
                        total: resp.total,
                        per_page: PAGE_SIZE,
                        on_page_change: move |next| page.set(next),
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
