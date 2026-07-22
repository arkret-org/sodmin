pub use arkret_core::models::{
    DirectoryListHandlesForSubjectRequestBody as ListHandlesForSubjectRequest,
    DirectorySubjectHandleList, HandleBindingState, HandleClaim,
};
pub use arkret_models_discovery::ops::HardeningStatus;
use serde::{Deserialize, Serialize};

// ── Pagination ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub total: Option<u64>,
    /// Opaque cursor for the next page. `None` when the current page is
    /// the last one.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

impl<T> ListResponse<T> {
    pub fn total_or_page_floor(&self, page: u64, per_page: u64) -> u64 {
        self.total.unwrap_or_else(|| {
            let loaded_until = page
                .saturating_sub(1)
                .saturating_mul(per_page)
                .saturating_add(self.data.len() as u64);
            if self.next_cursor.is_some() {
                loaded_until.saturating_add(1)
            } else {
                loaded_until
            }
        })
    }
}
