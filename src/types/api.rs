pub use cokret_core::ops::HardeningStatus;
pub use cokret_core::model::{
    DirectoryListHandlesForSubjectRequestBody as ListHandlesForSubjectRequest,
    DirectorySubjectHandleList, HandleBindingState, HandleClaim, MemberDeliveryBinding,
};
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
    pub fn total_or_len(&self) -> u64 {
        self.total.unwrap_or(self.data.len() as u64)
    }
}
