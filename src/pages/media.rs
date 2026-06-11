use dioxus::prelude::*;

use crate::api::media;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn MediaList() -> Element {
    let mut search = use_signal(String::new);
    let mut page = use_signal(|| 1u64);

    let page_val = *page.read();

    let stats = use_resource(|| async { media::get_media_statistics().await });

    let mut media_data =
        use_resource(move || async move { media::list_actor_media(page_val, PAGE_SIZE).await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("media.title"),
                description: t("media.subtitle"),
            }

            match &*stats.read() {
                Some(Ok(s)) => rsx! {
                    div { class: "grid gap-4 md:grid-cols-3",
                        Card {
                            CardContent { class: "p-4".to_string(),
                                p { class: "text-sm text-muted-foreground", {t("media.total_blobs")} }
                                p { class: "text-2xl font-bold", "{s.total_blobs}" }
                            }
                        }
                        Card {
                            CardContent { class: "p-4".to_string(),
                                p { class: "text-sm text-muted-foreground", {t("media.total_size")} }
                                p { class: "text-2xl font-bold", {format_bytes(s.total_size)} }
                            }
                        }
                        Card {
                            CardContent { class: "p-4".to_string(),
                                p { class: "text-sm text-muted-foreground", {t("media.quarantined")} }
                                p { class: "text-2xl font-bold", "{s.quarantined_count}" }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|body| body.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                    }
                },
                _ => rsx! {},
            }

            SearchInput {
                placeholder: t("media.search"),
                value: search.read().clone(),
                oninput: move |evt: FormEvent| {
                    search.set(evt.value());
                    page.set(1);
                },
            }

            match &*media_data.read() {
                Some(Ok(data)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("media.actor_id")} }
                                    TableHead { {t("media.display_name")} }
                                    TableHead { {t("media.blob_count")} }
                                    TableHead { {t("media.total_size")} }
                                }
                            }
                            TableBody {
                                if data.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("media.no_media")}
                                        }
                                    }
                                } else {
                                    for stat in data.data.iter() {
                                        {
                                            let actor_id = stat.actor_id.clone();
                                            let display_name = stat.display_name.clone().unwrap_or_else(|| "-".to_string());
                                            let blob_count = stat.blob_count;
                                            let total_size = format_bytes(stat.total_size);

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{actor_id}" }
                                                    TableCell { "{display_name}" }
                                                    TableCell { "{blob_count}" }
                                                    TableCell { "{total_size}" }
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
                        total: data.total_or_len(),
                        per_page: PAGE_SIZE,
                        on_page_change: move |p| page.set(p),
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|body| body.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| media_data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
