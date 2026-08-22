//! DTO adapters for the multi-sig partial-signature admin surface.
//!
//! All DTOs come from `soland-contracts`; this module only re-exports them
//! under the sodmin `types` tree.

pub use soland_contracts::admin::seal::{MultisigPendingEntry, MultisigPendingOutcome};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_row_helpers_come_from_shared_type() {
        let row = MultisigPendingEntry {
            seal_id:
                "ak:seal:sha256:6c60708bf5af164178744ae2ddfb641cbf9db8034a20aa0890be611efceb38bc"
                    .to_owned(),
            threshold_k: 3,
            threshold_n: 5,
            collected_partials: 1,
            ..Default::default()
        };

        assert_eq!(
            row.seal_id,
            "ak:seal:sha256:6c60708bf5af164178744ae2ddfb641cbf9db8034a20aa0890be611efceb38bc"
        );
        assert_eq!(row.remaining(), 2);
        assert_eq!(row.threshold_label(), "3 of 5");
        assert!(!row.is_threshold_met());
    }
}
