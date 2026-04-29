use dioxus::prelude::*;

use crate::api::spaces;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::icons::Icon;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn SpaceShow(space_id: String) -> Element {
    let space_id_data = space_id.clone();
    let space_id_members = space_id.clone();
    let mut data = use_resource(move || {
        let id = space_id_data.clone();
        async move { spaces::get_space(&id).await }
    });
    let members = use_resource(move || {
        let id = space_id_members.clone();
        async move { spaces::list_space_members(&id).await }
    });

    let space_id_for_block = space_id.clone();
    let space_id_for_unblock = space_id.clone();
    let space_id_for_delete = space_id.clone();

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(space)) => {
                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("nav.spaces"), route: Some(Route::SpaceList {}) },
                        BreadcrumbItem { label: space.name.as_deref().unwrap_or(&space.id).to_string(), route: None },
                    ];
                    rsx! {
                        Breadcrumbs {
                            items: breadcrumbs,
                        }

                        PageHeader {
                            title: space.name.as_deref().unwrap_or(&space.id).to_string(),
                        }

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("spaces.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                    {field_row(t("spaces.id"), space.id.clone())}
                                    {field_row(t("spaces.type"), space.space_type.as_deref().unwrap_or("default").to_string())}
                                    {field_row(t("spaces.discoverability"), space.discoverability.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("spaces.creator"), space.creator_id.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("spaces.members"), space.member_count.to_string())}
                                    {field_row(t("spaces.encrypted"), if space.is_encrypted { t("common.yes") } else { t("common.no") })}
                                    {field_row(t("spaces.join_rule"), space.join_rule.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("spaces.status"), if space.is_blocked { t("spaces.blocked") } else { t("spaces.active") })}
                                    {field_row(t("spaces.created_at"), space.created_at.as_deref().unwrap_or("-").to_string())}
                                        if let Some(ref topic) = space.topic {
                                            div { class: "pt-2",
                                                p { class: "text-sm text-muted-foreground mb-1", {t("spaces.topic")} }
                                                p { class: "text-sm", "{topic}" }
                                            }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("spaces.actions")} } }
                                CardContent {
                                    div { class: "space-y-2",
                                        if !space.is_blocked {
                                            Button {
                                                class: "w-full".to_string(),
                                                onclick: move |_| {
                                                    let sid = space_id_for_block.clone();
                                                    spawn(async move {
                                                        let _ = spaces::block_space(&sid).await;
                                                        data.restart();
                                                    });
                                                },
                                                {t("spaces.block")}
                                            }
                                        } else {
                                            Button {
                                                class: "w-full".to_string(),
                                                onclick: move |_| {
                                                    let sid = space_id_for_unblock.clone();
                                                    spawn(async move {
                                                        let _ = spaces::unblock_space(&sid).await;
                                                        data.restart();
                                                    });
                                                },
                                                {t("spaces.unblock")}
                                            }
                                        }
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            class: "w-full".to_string(),
                                            onclick: move |_| {
                                                let sid = space_id_for_delete.clone();
                                                spawn(async move {
                                                    let _ = spaces::delete_space(&sid).await;
                                                });
                                            },
                                            {t("spaces.delete")}
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("spaces.members")} } }
                            CardContent {
                                match &*members.read() {
                                    Some(Ok(member_list)) => rsx! {
                                        Table {
                                            TableHeader {
                                                TableRow {
                                                    TableHead { {t("spaces.actor_id")} }
                                                    TableHead { {t("spaces.display_name")} }
                                                    TableHead { {t("spaces.role")} }
                                                    TableHead { {t("spaces.joined_at")} }
                                                }
                                            }
                                            TableBody {
                                                for member in member_list.iter() {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs", "{member.actor_id}" }
                                                        TableCell { {member.display_name.as_deref().unwrap_or("-")} }
                                                        TableCell { {member.role.as_deref().unwrap_or("member")} }
                                                        TableCell { {member.joined_at.as_deref().unwrap_or("-")} }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => rsx! { p { class: "text-muted-foreground", {t("common.loading")} } },
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { ErrorBanner { message: e.message.clone(), on_retry: move |_| data.restart() } },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

fn field_row(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 last:border-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm font-medium", "{value}" }
        }
    }
}
