use dioxus::prelude::*;

use crate::api::realms;
use crate::components::realm_classification_badge::RealmClassificationBadge;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn RealmShow(realm_id: String) -> Element {
    let realm_id_data = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_data.clone();
        async move { realms::get_realm(&id).await }
    });

    let realm_id_for_delete = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(realm)) => {
                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("nav.realms"), route: Some(Route::RealmList {}) },
                        BreadcrumbItem { label: realm.title.as_deref().unwrap_or(&realm.id).to_string(), route: None },
                    ];
                    rsx! {
                        Breadcrumbs {
                            items: breadcrumbs,
                        }

                        PageHeader {
                            title: realm.title.as_deref().unwrap_or(&realm.id).to_string(),
                            // P3A.6 — Principal Control vs Collaboration
                            // Realm badge. Hidden when the field is absent
                            // on older soland deployments.
                            if let Some(ref cls) = realm.realm_class {
                                RealmClassificationBadge { realm_class: cls.clone() }
                            }
                        }

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("realms.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                    {field_row(t("realms.id"), realm.id.clone())}
                                    {field_row(t("realms.type"), realm_type_label(realm))}
                                    {field_row(t("realms.discoverability"), realm.discoverability.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.creator"), realm.created_by.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.members"), realm.member_count.to_string())}
                                    {field_row(t("realms.encrypted"), if realm.is_encrypted { t("common.yes") } else { t("common.no") })}
                                    {field_row(t("realms.join_rule"), realm.join_rule.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.status"), if realm.is_blocked { t("realms.blocked") } else { t("realms.active") })}
                                    {field_row(t("realms.created_at"), realm.created_at.as_deref().unwrap_or("-").to_string())}
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
                                        // `ck.realm.destroy` is irreversible at the principal
                                        // server. Route through the dedicated destroy page
                                        // (five normative-bullet checkboxes + typed `DESTROY`)
                                        // instead of a one-click delete — the bare button
                                        // previously fired DELETE with zero confirmation.
                                        Link {
                                            to: Route::RealmDestroy { realm_id: realm_id_for_delete.clone() },
                                            class: "inline-flex w-full items-center justify-center rounded-md bg-destructive px-4 py-2 text-sm font-medium text-destructive-foreground hover:bg-destructive/90",
                                            {t("realms.delete")}
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

fn realm_type_label(realm: &crate::types::AdminRealm) -> String {
    realm
        .realm_class
        .as_deref()
        .unwrap_or("collaboration")
        .to_owned()
}

fn field_row(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 last:border-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm font-medium", "{value}" }
        }
    }
}
