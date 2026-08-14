use dioxus::prelude::*;

use crate::api::realms;
use crate::components::realm_classification_badge::RealmClassificationBadge;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::router::Route;
use crate::types::realms::AdminRealmExt;
use crate::utils::i18n::t;

#[component]
pub fn RealmShow(realm_id: String) -> Element {
    let realm_id_data = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_data.clone();
        async move { realms::get_realm(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(realm)) => {
                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("nav.realms"), route: Some(Route::RealmList {}) },
                        BreadcrumbItem { label: realm.title.clone(), route: None },
                    ];
                    rsx! {
                        Breadcrumbs {
                            items: breadcrumbs,
                        }

                        PageHeader {
                            title: realm.title.clone(),
                            if let Some(cls) = realm.realm_class {
                                RealmClassificationBadge { realm_class: cls }
                            }
                        }

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("realms.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                    {field_row(t("realms.id"), realm.id.clone())}
                                    {field_row(t("realms.type"), realm.type_label().to_owned())}
                                    {field_row(t("realms.discoverability"), realm.discoverability_label().unwrap_or_else(|| "-".to_string()))}
                                    {field_row(t("realms.creator"), realm.created_by.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.members"), realm.member_count.to_string())}
                                    {field_row(t("realms.encrypted"), if realm.is_encrypted { t("common.yes") } else { t("common.no") })}
                                    {field_row(t("realms.join_rule"), realm.join_rule_label().unwrap_or_else(|| "-".to_string()))}
                                    {field_row(t("realms.status"), if realm.is_blocked { t("realms.blocked") } else { t("realms.active") })}
                                    {field_row(t("realms.created_at"), realm.created_at_display().unwrap_or_else(|| "-".to_string()))}
                                        if let Some(ref topic) = realm.topic {
                                            div { class: "pt-2",
                                                p { class: "text-sm text-muted-foreground mb-1", {t("realms.topic")} }
                                                p { class: "text-sm", "{topic}" }
                                            }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("realms.actions")} } }
                                CardContent {
                                    div { class: "space-y-2",
                                        // SOD-ORG-01..03 — verified organization
                                        // relationship + principal control audit.
                                        Link {
                                            to: Route::RealmOrganization { realm_id: realm.id.clone() },
                                            class: "inline-flex w-full items-center justify-center rounded-md border border-border px-4 py-2 text-sm font-medium hover:bg-muted",
                                            {t("realm_organization.title")}
                                        }
                                    }
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
