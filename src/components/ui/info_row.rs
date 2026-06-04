use dioxus::prelude::*;

/// Label/value field row shared by the detail (`show`) pages.
///
/// Replaces the identical local `fn InfoRow` that several detail pages were
/// each defining (`agents/show`, `federation/show`, `reports/show`): a muted
/// label on the left and a right-aligned, break-all value on the right.
#[component]
pub fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2",
            span { class: "text-sm font-medium text-muted-foreground", "{label}" }
            span { class: "text-sm max-w-[60%] text-right break-all", "{value}" }
        }
    }
}
