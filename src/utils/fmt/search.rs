//! Client-side name-or-id substring filter.
//!
//! Used by list pages as a fallback when the backend hasn't yet plumbed
//! the `filter[name_or_id]` query parameter. Comparison is lower-cased
//! and substring-based — admins typing "acme" should find an entry
//! named "Acme Corp" or id `acme-bot-12`.

/// `true` when either of the two source strings contains the query as
/// a case-insensitive substring. An empty `query` matches everything.
pub fn matches_name_or_id(query: &str, id: &str, name: Option<&str>) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    if id.to_lowercase().contains(&q) {
        return true;
    }
    if let Some(n) = name
        && n.to_lowercase().contains(&q)
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything() {
        assert!(matches_name_or_id("", "anything", None));
        assert!(matches_name_or_id("  ", "x", Some("y")));
    }

    #[test]
    fn matches_id_case_insensitive() {
        assert!(matches_name_or_id("ACME", "acme-bot-1", None));
        assert!(matches_name_or_id("bot", "acme-bot-1", None));
        assert!(!matches_name_or_id("zzz", "acme-bot-1", None));
    }

    #[test]
    fn matches_name_when_id_does_not() {
        assert!(matches_name_or_id("Corp", "id-1", Some("Acme Corp")));
        assert!(!matches_name_or_id("Corp", "id-1", None));
    }
}
