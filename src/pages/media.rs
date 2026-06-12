use dioxus::prelude::*;

use crate::api::media;
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
    let needle = search.read().to_ascii_lowercase();

    let mut stats_data = use_resource(move || async move { media::get_media_statistics().await });
    let mut actor_media_data =
        use_resource(move || async move { media::list_media_by_actor().await });
    let mut media_data =
        use_resource(move || async move { media::list_media(page_val, PAGE_SIZE).await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("media.title"),
                description: t("media.subtitle"),
            }

            match &*stats_data.read() {
                Some(Ok(stats)) => rsx! {
                    div { class: "grid gap-3 md:grid-cols-4",
                        {metric_tile("Blobs", stats.total_blobs.to_string())}
                        {metric_tile("Stored", format_bytes(stats.total_size))}
                        {metric_tile("Encrypted", stats.encrypted_count.to_string())}
                        {metric_tile("Quarantined", stats.quarantined_count.to_string())}
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|body| body.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| stats_data.restart(),
                    }
                },
                None => rsx! {
                    div { class: "grid gap-3 md:grid-cols-4",
                        for _ in 0..4 {
                            div { class: "h-20 rounded-md border bg-muted/30" }
                        }
                    }
                },
            }

            match &*actor_media_data.read() {
                Some(Ok(resp)) => actor_media_section(resp),
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|body| body.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| actor_media_data.restart(),
                    }
                },
                None => rsx! { div { class: "h-24 rounded-md border bg-muted/30" } },
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
                Some(Ok(data)) => {
                    let rows = data
                        .data
                        .iter()
                        .filter(|row| media_row_matches(row, &needle))
                        .cloned()
                        .collect::<Vec<_>>();
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { "Filename" }
                                        TableHead { "Type" }
                                        TableHead { "Realm" }
                                        TableHead { "Uploaded by" }
                                        TableHead { "Size" }
                                        TableHead { "Encrypted" }
                                        TableHead { "Created" }
                                    }
                                }
                                TableBody {
                                    if rows.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("media.no_media")}
                                            }
                                        }
                                    } else {
                                        for row in rows.iter() {
                                            {
                                                let filename = row.filename.clone().unwrap_or_else(|| "-".to_string());
                                                let media_type = row.media_type.clone().unwrap_or_else(|| "-".to_string());
                                                let realm_id = row.realm_id.clone().unwrap_or_else(|| "-".to_string());
                                                let uploaded_by = row.uploaded_by.clone().unwrap_or_else(|| "-".to_string());
                                                let size = format_bytes(row.size_bytes);
                                                let encrypted = if row.encrypted { "yes" } else { "no" };
                                                let created = row.created_at.clone().unwrap_or_else(|| "-".to_string());
                                                rsx! {
                                                    TableRow {
                                                        TableCell { "{filename}" }
                                                        TableCell { "{media_type}" }
                                                        TableCell { class: "font-mono text-xs max-w-[220px] truncate".to_string(), "{realm_id}" }
                                                        TableCell { class: "font-mono text-xs max-w-[220px] truncate".to_string(), "{uploaded_by}" }
                                                        TableCell { "{size}" }
                                                        TableCell { "{encrypted}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{created}" }
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

fn metric_tile(label: &str, value: String) -> Element {
    rsx! {
        div { class: "rounded-md border p-3",
            div { class: "text-xs uppercase text-muted-foreground", "{label}" }
            div { class: "mt-1 text-lg font-semibold", "{value}" }
        }
    }
}

fn actor_media_section(
    resp: &crate::types::ListResponse<crate::types::ActorMediaStatistics>,
) -> Element {
    rsx! {
        div { class: "rounded-md border",
            Table {
                TableHeader {
                    TableRow {
                        TableHead { "Actor" }
                        TableHead { "Display name" }
                        TableHead { "Blobs" }
                        TableHead { "Stored" }
                    }
                }
                TableBody {
                    if resp.data.is_empty() {
                        TableRow {
                            TableCell { class: "text-center text-muted-foreground py-6".to_string(), colspan: 99,
                                {t("media.no_media")}
                            }
                        }
                    } else {
                        for row in resp.data.iter() {
                            {
                                let actor_id = row.actor_id.clone();
                                let display_name = row.display_name.clone().unwrap_or_else(|| "-".to_string());
                                let blob_count = row.blob_count.to_string();
                                let total_size = format_bytes(row.total_size);
                                rsx! {
                                    TableRow {
                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{actor_id}" }
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
    }
}

fn media_row_matches(row: &crate::types::MediaRow, needle: &str) -> bool {
    if needle.trim().is_empty() {
        return true;
    }
    [
        row.filename.as_deref(),
        row.media_type.as_deref(),
        row.realm_id.as_deref(),
        row.uploaded_by.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| value.to_ascii_lowercase().contains(needle))
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
