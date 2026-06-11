use dioxus::prelude::*;

use crate::api::federation;
use crate::components::ui::auto_refresh::{self, AutoRefreshPicker, RefreshInterval};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::fmt::search::matches_name_or_id;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;
const AUTOREFRESH_STORAGE_KEY: &str = "sodmin.federation.autorefresh";

#[component]
pub fn FederationList() -> Element {
    let mut peer_cursors = use_signal(|| vec![None::<String>]);
    let mut search = use_signal(String::new);
    let mut autorefresh = use_signal(|| auto_refresh::load(AUTOREFRESH_STORAGE_KEY));

    let peer_cursor = peer_cursors.read().last().cloned().unwrap_or(None);
    let search_val = search.read().clone();

    let mut peers = use_resource(move || {
        let c = peer_cursor.clone();
        let s = search_val.clone();
        async move { federation::list_federation_peers(c.as_deref(), PAGE_SIZE, &s).await }
    });

    let mut interval_handle = use_signal::<Option<gloo_timers::callback::Interval>>(|| None);
    use_effect(move || {
        let choice = *autorefresh.read();
        interval_handle.set(None);
        if let Some(ms) = choice.millis() {
            let mut peers = peers;
            let handle = gloo_timers::callback::Interval::new(ms, move || {
                peers.restart();
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
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| peers.restart(),
                    {t("common.refresh")}
                }
            }

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
                                                        TableCell { Badge { variant: BadgeVariant::Secondary, "{status}" } }
                                                        TableCell { "{trust_level}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{last_txn}" }
                                                        TableCell { class: "max-w-[240px] truncate text-muted-foreground".to_string(), "{last_error}" }
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
}
