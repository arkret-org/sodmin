use dioxus::prelude::*;

use crate::api::actors;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::icons::Icon;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn ActorList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut search = use_signal(String::new);
    let per_page: u64 = 20;

    let mut data =
        use_resource(
            move || async move { actors::list_actors(page(), per_page, &search.read()).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("nav.actors"),
                description: t("actors.description"),
            }

            div { class: "flex items-center gap-4",
                SearchInput {
                    value: search(),
                    placeholder: t("actors.search_placeholder"),
                    oninput: move |evt: FormEvent| { search.set(evt.value()); page.set(1); },
                }
                Link {
                    to: Route::ActorCreate {},
                    class: "inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    Icon { name: "plus".to_string(), class: "h-4 w-4".to_string() }
                    {t("actors.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-lg border glass-panel overflow-hidden",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("actors.id")} }
                                    TableHead { {t("actors.handle")} }
                                    TableHead { {t("actors.display_name")} }
                                    TableHead { {t("actors.status")} }
                                    TableHead { {t("actors.is_admin")} }
                                    TableHead { {t("actors.created_at")} }
                                }
                            }
                            TableBody {
                                for actor in resp.data.iter() {
                                    {
                                        let actor_id = actor.id.clone();
                                        rsx! {
                                            TableRow {
                                                TableCell { class: "font-mono text-xs",
                                                    Link {
                                                        to: Route::ActorShow { actor_id: actor_id.clone() },
                                                        class: "hover:underline",
                                                        "{actor.id}"
                                                    }
                                                }
                                                TableCell { {actor.handle.as_deref().unwrap_or("-")} }
                                                TableCell { {actor.display_name.as_deref().unwrap_or("-")} }
                                                TableCell {
                                                    if actor.is_suspended {
                                                        {t("actors.suspended")}
                                                    } else if actor.is_deactivated {
                                                        {t("actors.deactivated")}
                                                    } else {
                                                        {t("actors.active")}
                                                    }
                                                }
                                                TableCell {
                                                    if actor.is_admin {
                                                        Icon { name: "check".to_string(), class: "h-4 w-4 text-green-500".to_string() }
                                                    }
                                                }
                                                TableCell { {actor.created_at.as_deref().unwrap_or("-")} }
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
                        total: resp.total_or_len(),
                        on_page_change: move |p: u64| page.set(p),
                    }
                },
                Some(Err(e)) => rsx! { ErrorBanner { message: e.message.clone(), on_retry: move |_| data.restart() } },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
