/// Format an epoch-millisecond timestamp for display. Rendered in the
/// browser's **local** timezone (matching [`format_iso_datetime`]) so two
/// time columns on the same page can't silently differ by the UTC offset.
pub fn format_timestamp(ts_ms: u64) -> String {
    if ts_ms == 0 {
        return "-".to_string();
    }
    let secs = (ts_ms / 1000) as i64;
    chrono::DateTime::from_timestamp(secs, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_iso_datetime(value: &str) -> String {
    if value.trim().is_empty() {
        return "-".to_string();
    }

    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&chrono::Local))
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&format!("{value}Z"))
                .map(|dt| dt.with_timezone(&chrono::Local))
        });

    match parsed {
        Ok(dt) => match crate::utils::i18n::current_language() {
            crate::utils::i18n::Language::ZhCn => dt.format("%Y年%m月%d日 %H:%M:%S").to_string(),
            crate::utils::i18n::Language::En => dt.format("%b %-d, %Y, %-I:%M:%S %p").to_string(),
        },
        Err(_) => value.to_string(),
    }
}
