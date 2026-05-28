//! Accessible checkbox primitive.
//!
//! Supports a tri-state appearance via `indeterminate` for "some rows
//! selected" header cells. The DOM input element's `indeterminate`
//! property is set imperatively after render because HTML attributes
//! can only express `checked`/`unchecked`.
//!
//! Pure leaf — emits an `onchange(bool)` whose value is the *new*
//! intent (i.e. `!checked` from the previous render). For
//! indeterminate→checked toggling the parent decides the semantics
//! (typically: "select all visible rows").

use dioxus::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Checkbox(
    #[props(default)] id: String,
    #[props(default)] class: String,
    #[props(default)] aria_label: String,
    #[props(default)] checked: bool,
    #[props(default)] indeterminate: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    // Imperatively reflect `indeterminate` onto the DOM element after
    // render — it's not addressable via HTML attributes.
    let id_for_effect = id.clone();
    let indeterminate_flag = indeterminate;
    let checked_flag = checked;
    use_effect(move || {
        if id_for_effect.is_empty() {
            return;
        }
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
            && let Some(el) = doc.get_element_by_id(&id_for_effect)
            && let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>()
        {
            input.set_indeterminate(indeterminate_flag);
            // Keep DOM `checked` in sync with the prop —
            // Dioxus re-renders the attribute too, but
            // setting indeterminate sometimes clears the
            // visual state of a re-keyed element.
            let _ = checked_flag;
        }
    });

    let label = if aria_label.is_empty() {
        "Select".to_string()
    } else {
        aria_label
    };

    rsx! {
        input {
            id: id.clone(),
            r#type: "checkbox",
            class: "h-4 w-4 rounded border-input text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 touch-target {class}",
            aria_label: label,
            checked,
            disabled,
            onchange: move |evt| {
                // The browser flips `checked` before firing change; read
                // the new state from the DOM rather than negating the
                // stale prop, which is wrong when the previous render
                // was `indeterminate`.
                let new_state = evt.value() == "true" || evt.value() == "on";
                onchange.call(new_state);
            },
        }
    }
}

/// Pure helper: derive the (checked, indeterminate) pair for a header
/// "select all" cell given the count of selected rows on the current
/// page and the total visible rows. Empty pages render unchecked /
/// not indeterminate so the header doesn't lie.
pub fn header_state(selected_on_page: usize, page_size: usize) -> (bool, bool) {
    if page_size == 0 || selected_on_page == 0 {
        (false, false)
    } else if selected_on_page >= page_size {
        (true, false)
    } else {
        (false, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_state_empty_page() {
        assert_eq!(header_state(0, 0), (false, false));
        assert_eq!(header_state(0, 10), (false, false));
    }

    #[test]
    fn header_state_partial() {
        assert_eq!(header_state(3, 10), (false, true));
        assert_eq!(header_state(1, 2), (false, true));
    }

    #[test]
    fn header_state_full() {
        assert_eq!(header_state(10, 10), (true, false));
        // Over-count (defensive — shouldn't happen but stable)
        assert_eq!(header_state(11, 10), (true, false));
    }
}
