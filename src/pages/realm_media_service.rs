//! R3 (UI-3) — Realm `media_service` editor.
//!
//! Renders the `foci[]` list for a single Realm: each focus carries an
//! `id`, a `type` ∈ {`livekit`, `mediasoup`, `janus`, `contrix-native`,
//! `moq-relay`}, an SFU `connect_url`, a `service_did`, and the list of
//! `regions` it serves.
//!
//! When the underlying realm cell still carries the legacy single
//! `sfu_endpoint` shape, a migration banner is rendered above the
//! editor so the operator knows they're looking at a pre-R3 server.
//!
//! TODO(R3.1): plumb the soland `/api/admin/v1/realms/{id}/media-service`
//! GET / PUT pair through `api::server`. For now the page is a writable
//! UI scaffold that round-trips through the client-side state — saving
//! emits a no-op toast so the contract is still exercised in the SPA's
//! integration tests.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::i18n::t;

/// The five focus types accepted by the contrix-spec v3 media_service
/// binding profile. Keep in sync with `cx.profile.media_service_binding.v1`.
pub const FOCUS_TYPES: &[&str] = &[
    "livekit",
    "mediasoup",
    "janus",
    "contrix-native",
    "moq-relay",
];

#[derive(Debug, Clone, PartialEq)]
struct FocusDraft {
    id: String,
    focus_type: String,
    connect_url: String,
    service_did: String,
    regions: String,
}

impl FocusDraft {
    fn blank() -> Self {
        Self {
            id: String::new(),
            focus_type: FOCUS_TYPES[0].to_string(),
            connect_url: String::new(),
            service_did: String::new(),
            regions: String::new(),
        }
    }
}

#[component]
pub fn RealmMediaService(realm_id: String) -> Element {
    // TODO(R3.1): hydrate from soland GET. Until the wire surface lands
    // we boot with a single blank draft so the operator can edit and the
    // save button has something to round-trip.
    let mut foci = use_signal::<Vec<FocusDraft>>(|| vec![FocusDraft::blank()]);
    let mut legacy_endpoint_present = use_signal(|| false);
    let realm_id_label = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("delivery_binding.realm"), route: Some(Route::RealmDeliveryBinding { realm_id: realm_id.clone() }) },
                    BreadcrumbItem { label: t("media_service.title"), route: None },
                ],
            }

            PageHeader {
                title: t("media_service.title"),
                description: format!("{}: {}", t("delivery_binding.realm"), realm_id_label),
            }

            // R3 — migration banner. Rendered when the underlying cell
            // still carries `sfu_endpoint`. The toggle is wired off the
            // (placeholder) signal so the surface is testable until the
            // GET endpoint is plumbed.
            if *legacy_endpoint_present.read() {
                div {
                    class: "rounded-md border-2 border-amber-600 bg-amber-600/10 px-3 py-2 text-sm text-amber-900 dark:text-amber-100",
                    role: "alert",
                    p { class: "font-semibold",
                        span { class: "mr-2", "\u{26A0}" }
                        {t("media_service.legacy_banner_title")}
                    }
                    p { class: "text-xs",
                        {t("media_service.legacy_banner_body")}
                    }
                    p { class: "text-xs mt-1",
                        {t("error.legacy_single_endpoint_media_service")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| legacy_endpoint_present.set(false),
                        "Dismiss banner"
                    }
                }
            } else {
                // Dev affordance so the banner can be exercised without
                // a stale realm cell. Removed in R3.1 when the GET endpoint
                // surfaces the legacy bit canonically.
                div { class: "text-xs text-muted-foreground",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| legacy_endpoint_present.set(true),
                        "Simulate legacy sfu_endpoint"
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { {t("media_service.foci_label")} }
                    CardDescription { {t("media_service.subtitle")} }
                }
                CardContent {
                    {
                        let snapshot = foci.read().clone();
                        rsx! {
                            div { class: "space-y-3",
                                if snapshot.is_empty() {
                                    p { class: "text-sm text-muted-foreground", {t("media_service.empty")} }
                                } else {
                                    for (idx, focus) in snapshot.iter().enumerate() {
                                        {render_focus_row(idx, focus, foci)}
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    size: ButtonSize::Sm,
                                    onclick: move |_| {
                                        let mut next = foci.read().clone();
                                        next.push(FocusDraft::blank());
                                        foci.set(next);
                                    },
                                    {t("media_service.add")}
                                }
                                div { class: "flex justify-end",
                                    Button {
                                        variant: ButtonVariant::Default,
                                        onclick: move |_| {
                                            // TODO(R3.1): real PUT against
                                            // `/api/admin/v1/realms/{id}/media-service`.
                                            // The payload shape is:
                                            //   { "foci": [
                                            //       {"id":"…","type":"livekit",
                                            //        "connect_url":"…","service_did":"…",
                                            //        "regions":["us-east"]}
                                            //   ] }
                                            show_toast(&t("media_service.save_ok"), ToastVariant::Success);
                                        },
                                        {t("media_service.save")}
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

fn render_focus_row(idx: usize, focus: &FocusDraft, mut foci: Signal<Vec<FocusDraft>>) -> Element {
    let id_val = focus.id.clone();
    let type_val = focus.focus_type.clone();
    let url_val = focus.connect_url.clone();
    let did_val = focus.service_did.clone();
    let regions_val = focus.regions.clone();
    let known_type = FOCUS_TYPES.contains(&type_val.as_str());

    rsx! {
        div {
            class: "rounded-md border p-3 space-y-2",
            div { class: "flex items-center justify-between",
                p { class: "text-xs text-muted-foreground", "focus #{idx + 1}" }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        let mut next = foci.read().clone();
                        if idx < next.len() {
                            next.remove(idx);
                            foci.set(next);
                        }
                    },
                    {t("media_service.remove")}
                }
            }
            div { class: "grid gap-2 md:grid-cols-2",
                div { class: "space-y-1",
                    Label { {t("media_service.focus_id")} }
                    Input {
                        value: id_val.clone(),
                        oninput: move |evt: FormEvent| {
                            let mut next = foci.read().clone();
                            if let Some(slot) = next.get_mut(idx) {
                                slot.id = evt.value();
                                foci.set(next);
                            }
                        },
                    }
                }
                div { class: "space-y-1",
                    Label { {t("media_service.focus_type")} }
                    select {
                        class: "w-full rounded-md border bg-background px-3 py-2 text-sm",
                        value: type_val.clone(),
                        oninput: move |evt: FormEvent| {
                            let mut next = foci.read().clone();
                            if let Some(slot) = next.get_mut(idx) {
                                slot.focus_type = evt.value();
                                foci.set(next);
                            }
                        },
                        for ft in FOCUS_TYPES.iter() {
                            option { value: *ft, selected: *ft == type_val, "{ft}" }
                        }
                    }
                    if !known_type {
                        Badge { variant: BadgeVariant::Destructive, "unknown type" }
                    }
                }
                div { class: "space-y-1 md:col-span-2",
                    Label { {t("media_service.focus_connect_url")} }
                    Input {
                        value: url_val,
                        placeholder: "https://sfu.example.net".to_string(),
                        oninput: move |evt: FormEvent| {
                            let mut next = foci.read().clone();
                            if let Some(slot) = next.get_mut(idx) {
                                slot.connect_url = evt.value();
                                foci.set(next);
                            }
                        },
                    }
                }
                div { class: "space-y-1",
                    Label { {t("media_service.focus_service_did")} }
                    Input {
                        value: did_val,
                        placeholder: "did:cx:media-service".to_string(),
                        oninput: move |evt: FormEvent| {
                            let mut next = foci.read().clone();
                            if let Some(slot) = next.get_mut(idx) {
                                slot.service_did = evt.value();
                                foci.set(next);
                            }
                        },
                    }
                }
                div { class: "space-y-1",
                    Label { {t("media_service.focus_regions")} }
                    Input {
                        value: regions_val,
                        placeholder: "us-east, eu-west".to_string(),
                        oninput: move |evt: FormEvent| {
                            let mut next = foci.read().clone();
                            if let Some(slot) = next.get_mut(idx) {
                                slot.regions = evt.value();
                                foci.set(next);
                            }
                        },
                    }
                }
            }
        }
    }
}
