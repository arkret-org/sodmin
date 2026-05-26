use dioxus::prelude::*;

use crate::api::spaces;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::icons::Icon;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::csv::{build_csv, export_to_csv};
use crate::utils::i18n::t;

#[component]
pub fn SpaceList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut search = use_signal(String::new);
    let per_page: u64 = 20;

    let mut data =
        use_resource(
            move || async move { spaces::list_spaces(page(), per_page, &search.read()).await },
        );

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

            div { class: "flex items-center gap-4",
                SearchInput {
                    value: search(),
                    placeholder: t("spaces.search_placeholder"),
                    oninput: move |evt: FormEvent| { search.set(evt.value()); page.set(1); },
                }
                Link {
                    to: Route::SpaceCreate {},
                    class: "inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    Icon { name: "plus".to_string(), class: "h-4 w-4".to_string() }
                    {t("spaces.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
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
                                for space in resp.data.iter() {
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
                    Pagination {
                        page: page(),
                        per_page,
                        total: resp.total,
                        on_page_change: move |p: u64| page.set(p),
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
