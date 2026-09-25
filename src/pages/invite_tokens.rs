use dioxus::prelude::*;

use crate::api::invite_tokens;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn InviteTokenList() -> Element {
    let mut page = use_signal(|| 1u64);

    let page_val = *page.read();

    let mut data = use_resource(move || async move {
        invite_tokens::list_invite_tokens(page_val, PAGE_SIZE).await
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("invite_tokens.title"),
                description: t("invite_tokens.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("invite_tokens.id")} }
                                    TableHead { {t("invite_tokens.invitee")} }
                                    TableHead { {t("invite_tokens.status")} }
                                    TableHead { {t("invite_tokens.uses_allowed")} }
                                    TableHead { {t("invite_tokens.uses_completed")} }
                                    TableHead { {t("invite_tokens.uses_pending")} }
                                    TableHead { {t("invite_tokens.expires_at")} }
                                    TableHead { {t("invite_tokens.realm_id")} }
                                    TableHead { {t("invite_tokens.created_at")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("invite_tokens.no_tokens")}
                                        }
                                    }
                                } else {
                                    for token in resp.data.iter() {
                                        {
                                            let id = token.id.clone();
                                            let invitee = token
                                                .invitee_id
                                                .as_ref()
                                                .map(ToString::to_string)
                                                .unwrap_or_else(|| "-".to_string());
                                            let status = token.status.clone();
                                            let ua = token.uses_allowed.to_string();
                                            let uc = token.uses_completed.to_string();
                                            let up = token.uses_pending.to_string();
                                            let exp = token.expires_at.as_ref().map(chrono::DateTime::to_rfc3339).unwrap_or_else(|| "-".to_string());
                                            let rid = token.realm_id.clone();
                                            let created = token.created_at.to_rfc3339();

                                            rsx! {
                                                TableRow {
                                                    key: "{id}",
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[200px] truncate font-mono text-xs".to_string(), "{invitee}" }
                                                    TableCell { "{status}" }
                                                    TableCell { "{ua}" }
                                                    TableCell { "{uc}" }
                                                    TableCell { "{up}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{exp}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{rid}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(total) = resp.total {
                        Pagination {
                            page: page_val,
                            total,
                            per_page: PAGE_SIZE,
                            on_page_change: move |p| page.set(p),
                        }
                    } else {
                        p { class: "text-sm text-muted-foreground", {t("pagination.total_unknown")} }
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
