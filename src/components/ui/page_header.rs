use dioxus::prelude::*;
// `PageHeader` 已迁移到 yoface(签名一致,渲染走 css_module + Soft Orbit 令牌)。
// 直接 re-export,调用点零改动。
pub use yoface::ui::page_header::PageHeader;

use crate::router::Route;

// `Breadcrumbs` / `BreadcrumbItem` 保留本地:sodmin 用 `crate::router::Route`
// 强类型路由,yoface 版改用 `href: Option<String>`(共享库不假设下游路由)。
// 强类型在 sodmin 侧更安全(编译期校验路由),故保留本地实现,仅样式仍用
// 应用级 utility class。

#[derive(Debug, Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
}

#[component]
pub fn Breadcrumbs(items: Vec<BreadcrumbItem>) -> Element {
    rsx! {
        nav { class: "flex items-center space-x-1 text-sm text-muted-foreground mb-4",
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    span { class: "mx-1", "/" }
                }
                if let Some(ref route) = item.route {
                    Link {
                        to: route.clone(),
                        class: "hover:text-foreground transition-colors",
                        "{item.label}"
                    }
                } else {
                    span { class: "text-foreground font-medium", "{item.label}" }
                }
            }
        }
    }
}
