use dioxus::prelude::*;
// `PageHeader` 已迁移到 yoface(签名一致,渲染走 css_module + Soft Orbit 令牌)。
// 直接 re-export,调用点零改动。
pub use yoface::ui::page_header::PageHeader;

use crate::router::Route;

// Breadcrumbs 渲染已迁移到 yoface(`yoface::ui::page_header::Breadcrumbs`,
// `href: Option<String>`)。本地仅保留一层**薄适配器**:sodmin 调用点用
// `crate::router::Route` 强类型路由(编译期校验路由存在),适配器把每个
// `Route` 经 `Routable` 的 `Display`(`route.to_string()` 即 URL path)转成
// yoface 期望的 `href` 字符串,再委托 yoface 渲染。原手写的
// `nav`/`Link`/utility-class 实现已删除。
#[derive(Debug, Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
}

#[component]
pub fn Breadcrumbs(items: Vec<BreadcrumbItem>) -> Element {
    let mapped: Vec<yoface::ui::page_header::BreadcrumbItem> = items
        .into_iter()
        .map(|item| yoface::ui::page_header::BreadcrumbItem {
            label: item.label,
            href: item.route.map(|r| r.to_string()),
        })
        .collect();
    rsx! {
        yoface::ui::page_header::Breadcrumbs { items: mapped }
    }
}
