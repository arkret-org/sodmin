use dioxus::prelude::*;

/// `Input` — 已迁移到 yoface(`yoface::ui::input::Input`,css_module 渲染)。
///
/// 本地保留这一薄**适配器**:sodmin 调用点以命名 prop 传 `value` / `placeholder`
/// / `disabled` / `oninput`(非 `Option`)/ 以及一组 HTML 约束(`min_length` /
/// `max_length` / `pattern` / `aria_describedby` 等)。适配器把它们透传为 yoface
/// `Input` 的 attributes(`value` / `placeholder` 等都是 input 全局属性),
/// `oninput` 包成 yoface 的 `Option<EventHandler>`。~未知数量调用点零改动。
#[component]
pub fn Input(
    #[props(default)] id: String,
    #[props(default)] name: String,
    #[props(default = "text".to_string())] r#type: String,
    #[props(default)] class: String,
    #[props(default)] placeholder: String,
    #[props(default)] value: String,
    #[props(default)] aria_label: String,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    /// HTML `minlength` constraint. `None` omits the attribute so
    /// callers that don't care preserve the prior wire shape.
    #[props(default)]
    min_length: Option<u32>,
    /// HTML `maxlength` constraint.
    #[props(default)]
    max_length: Option<u32>,
    /// HTML `pattern` regex for client-side validation. The pattern
    /// is anchored by the browser so callers should not add `^`/`$`.
    #[props(default)]
    pattern: Option<String>,
    /// Optional ARIA description (e.g. inline error message id). When
    /// `pattern` validation fails the browser surfaces this via the
    /// standard `:invalid` pseudoclass; the description hint lets
    /// screen-reader users locate the message.
    #[props(default)]
    aria_describedby: String,
    #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element {
    let resolved_name = if name.is_empty() { id.clone() } else { name };
    rsx! {
        yoface::ui::input::Input {
            id,
            name: resolved_name,
            r#type,
            class: "{class}",
            placeholder,
            value,
            aria_label,
            aria_describedby,
            disabled,
            required,
            minlength: min_length.map(|n| n.to_string()),
            maxlength: max_length.map(|n| n.to_string()),
            pattern: pattern.clone(),
            oninput: move |evt| oninput.call(evt),
        }
    }
}

#[component]
pub fn Label(
    #[props(default)] class: String,
    #[props(default)] r#for: String,
    children: Element,
) -> Element {
    rsx! {
        label {
            r#for,
            class: "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 {class}",
            {children}
        }
    }
}

#[component]
pub fn SearchInput(
    #[props(default)] placeholder: String,
    #[props(default = "search".to_string())] label: String,
    value: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let input_label = if label.is_empty() {
        "Search".to_string()
    } else {
        label
    };
    rsx! {
        div { class: "relative",
            label {
                class: "sr-only",
                r#for: "sodmin-search-input",
                "{input_label}"
            }
            svg {
                class: "absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground",
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                circle { cx: "11", cy: "11", r: "8" }
                path { d: "m21 21-4.3-4.3" }
            }
            input {
                id: "sodmin-search-input",
                name: "search",
                r#type: "search",
                class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 pl-8 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 touch-target",
                placeholder: if placeholder.is_empty() { "Search...".to_string() } else { placeholder },
                value,
                aria_label: input_label,
                oninput: move |evt| oninput.call(evt),
            }
        }
    }
}
