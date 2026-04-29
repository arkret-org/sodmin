use dioxus::prelude::*;

use crate::api::devices;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn DeviceList() -> Element {
    let mut search = use_signal(String::new);
    let mut page = use_signal(|| 1u64);
    let mut show_delete_dialog = use_signal(|| None::<String>);

    let page_val = *page.read();
    let search_val = search.read().clone();

    let mut data = use_resource(move || {
        let s = search_val.clone();
        async move { devices::list_devices(page_val, PAGE_SIZE, &s).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("devices.title"),
                description: t("devices.subtitle"),
            }

            SearchInput {
                placeholder: t("devices.search"),
                value: search.read().clone(),
                oninput: move |evt: FormEvent| {
                    search.set(evt.value());
                    page.set(1);
                },
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
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
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("devices.no_devices")}
                                        }
                                    }
                                } else {
                                    for device in resp.data.iter() {
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

                                            rsx! {
                                                TableRow {
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

                    Pagination {
                        page: page_val,
                        total: resp.total,
                        per_page: PAGE_SIZE,
                        on_page_change: move |p| page.set(p),
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

        ConfirmDialog {
            open: show_delete_dialog.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this device?".to_string(),
            confirm_text: t("common.delete"),
            destructive: true,
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
