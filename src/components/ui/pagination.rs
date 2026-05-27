use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};

/// Cursor-based prev/next pair used by every list page that has been
/// migrated off legacy page+total pagination. The "page depth" is just
/// the size of the cursor stack the caller is maintaining (1 == first
/// page) — it's a UX affordance, not an O(N) computation. Going back
/// pops the stack rather than re-requesting from the server, so admins
/// flipping prev/next don't burn server pages.
#[component]
pub fn CursorPagination(
    /// 1-indexed depth of the cursor stack.
    depth: usize,
    /// `true` when a `next_cursor` is available on the current page.
    has_next: bool,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between px-2 py-4",
            div { class: "text-sm text-muted-foreground",
                {format!("Page {}", depth)}
            }
            div { class: "flex items-center space-x-2",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: depth <= 1,
                    onclick: move |_| on_prev.call(()),
                    "Previous"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: !has_next,
                    onclick: move |_| on_next.call(()),
                    "Next"
                }
            }
        }
    }
}

#[component]
pub fn Pagination(
    page: u64,
    total: u64,
    per_page: u64,
    on_page_change: EventHandler<u64>,
) -> Element {
    let total_pages = if total == 0 {
        1
    } else {
        total.div_ceil(per_page)
    };

    let from = if total == 0 {
        0
    } else {
        (page - 1) * per_page + 1
    };
    let to = std::cmp::min(page * per_page, total);

    rsx! {
        div { class: "flex items-center justify-between px-2 py-4",
            div { class: "text-sm text-muted-foreground",
                "Showing {from}-{to} of {total}"
            }
            div { class: "flex items-center space-x-2",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: page <= 1,
                    onclick: move |_| on_page_change.call(page - 1),
                    "Previous"
                }
                span { class: "text-sm text-muted-foreground",
                    "Page {page} of {total_pages}"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: page >= total_pages,
                    onclick: move |_| on_page_change.call(page + 1),
                    "Next"
                }
            }
        }
    }
}
