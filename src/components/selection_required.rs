use dioxus::prelude::*;

use crate::components::ui::empty_state::EmptyState;

pub fn is_placeholder_resource_id(id: &str) -> bool {
    id.trim() == "_"
}

pub fn selection_required_state(kind: &str) -> Element {
    let title = format!("Select a {kind}");
    let description = format!(
        "Open this page from a concrete {kind} detail view, or replace `_` in the URL with a real identifier."
    );
    rsx! {
        EmptyState {
            icon: "search".to_string(),
            title,
            description,
        }
    }
}
