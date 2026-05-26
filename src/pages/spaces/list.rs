use dioxus::prelude::*;

use crate::api::spaces;
use crate::components::ui::auto_refresh::{self, AutoRefreshPicker, RefreshInterval};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::icons::Icon;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::csv::{build_csv, export_to_csv};
use crate::utils::i18n::t;
use crate::utils::search::matches_name_or_id;

const PAGE_SIZE: u64 = 20;
const AUTOREFRESH_STORAGE_KEY: &str = "sodmin.spaces.autorefresh";

#[component]
pub fn SpaceList() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut search = use_signal(String::new);
    let mut autorefresh = use_signal(|| auto_refresh::load(AUTOREFRESH_STORAGE_KEY));

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let search_val = search.read().clone();

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        let s = search_val.clone();
        async move { spaces::list_spaces(cursor.as_deref(), PAGE_SIZE, &s).await }
    });

    let mut interval_handle = use_signal::<Option<gloo_timers::callback::Interval>>(|| None);
    use_effect(move || {
        let choice = *autorefresh.read();
        interval_handle.set(None);
        if let Some(ms) = choice.millis() {
            let mut data = data;
            let handle = gloo_timers::callback::Interval::new(ms, move || {
                data.restart();
            });
            interval_handle.set(Some(handle));
        }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("nav.spaces"),
                description: t("spaces.description"),
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        if let Some(Ok(resp)) = data.read().as_ref() {
                            let rows: Vec<Vec<String>> = resp.data.iter().map(|s| vec![
                                s.id.clone(),
                                s.name.clone().unwrap_or_default(),
                                s.space_type.clone().unwrap_or_default(),
                                s.member_count.to_string(),
                                if s.is_encrypted { "true".into() } else { "false".into() },
                                if s.is_blocked { "blocked".into() } else { "active".into() },
                                s.created_at.clone().unwrap_or_default(),
                            ]).collect();
                            let csv = build_csv(
                                &["id", "name", "space_type", "member_count", "encrypted", "status", "created_at"],
                                &rows,
                            );
                            export_to_csv("spaces.csv", &csv);
                            show_toast("Spaces CSV downloaded", ToastVariant::Success);
                        }
                    },
                    {t("common.export_csv")}
                }
            }

            div { class: "flex items-center gap-4 flex-wrap",
                div { class: "flex-1 min-w-[240px]",
                    SearchInput {
                        value: search(),
                        placeholder: t("spaces.search_placeholder"),
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
                Link {
                    to: Route::SpaceCreate {},
                    class: "inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    Icon { name: "plus".to_string(), class: "h-4 w-4".to_string() }
                    {t("spaces.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => {
                    let next_cursor = resp.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let search_for_filter = search.read().clone();
                    rsx! {
                        div { class: "rounded-lg border glass-panel overflow-hidden",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("spaces.id")} }
                                        TableHead { {t("spaces.name")} }
                                        TableHead { {t("spaces.type")} }
                                        TableHead { {t("spaces.members")} }
                                        TableHead { {t("spaces.encrypted")} }
                                        TableHead { {t("spaces.status")} }
                                        TableHead { {t("spaces.created_at")} }
                                    }
                                }
                                TableBody {
                                    for space in resp.data.iter().filter(|s| matches_name_or_id(&search_for_filter, &s.id, s.name.as_deref())) {
                                        {
                                            let sid = space.id.clone();
                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-mono text-xs",
                                                        Link {
                                                            to: Route::SpaceShow { space_id: sid.clone() },
                                                            class: "hover:underline",
                                                            "{space.id}"
                                                        }
                                                    }
                                                    TableCell { {space.name.as_deref().unwrap_or("-")} }
                                                    TableCell { {space.space_type.as_deref().unwrap_or("default")} }
                                                    TableCell { "{space.member_count}" }
                                                    TableCell {
                                                        if space.is_encrypted {
                                                            Icon { name: "lock".to_string(), class: "h-4 w-4 text-green-500".to_string() }
                                                        }
                                                    }
                                                    TableCell {
                                                        if space.is_blocked {
                                                            {t("spaces.blocked")}
                                                        } else {
                                                            {t("spaces.active")}
                                                        }
                                                    }
                                                    TableCell { {space.created_at.as_deref().unwrap_or("-")} }
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
        }
    }
}
