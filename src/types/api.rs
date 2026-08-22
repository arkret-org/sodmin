pub use arkret_models_discovery::ops::HardeningStatus;
pub use arkret_models_discovery::{
    DirectoryListHandlesForSubjectRequestBody, DirectorySubjectHandleList,
};
pub use arkret_models_identity::{HandleBindingState, HandleClaim};
use serde::{Deserialize, Serialize};

// ── Pagination ──

/// Permissive read view over the several list envelopes soland serves.
///
/// It is deliberately NOT a shared contract type: the producers differ
/// (`AdminCollectionOutcome` carries `resource` / `items` / `production_gap`
/// alongside `data`, the media by-actor endpoint carries `{data, total,
/// next_cursor}`, the recovery-policy endpoint nests its rows under
/// `policies`), and the client folds them into one shape. The *rows* inside
/// are contract-authoritative — that is where producer/consumer drift is
/// caught.
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
