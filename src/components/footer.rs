use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "border-t py-4 px-6 text-center text-xs text-muted-foreground",
            "sodmin"
            " | "
            "Contrix Admin"
        }
    }
}
