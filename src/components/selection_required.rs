use dioxus::prelude::*;

use crate::components::ui::empty_state::EmptyState;
use crate::utils::i18n::t;

pub fn is_placeholder_resource_id(id: &str) -> bool {
    id.trim() == "_"
}

pub fn selection_required_state(kind: &str) -> Element {
    let title = t("selection_required.title").replace("{kind}", kind);
    let description = t("selection_required.description").replace("{kind}", kind);
    rsx! {
        EmptyState {
            icon_name: "search".to_string(),
            title,
            description,
        }
    }
}
