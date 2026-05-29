use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CACHE: RefCell<HashMap<String, (f64, String)>> = RefCell::new(HashMap::new());
}

pub fn get_cached(key: &str, ttl_ms: f64) -> Option<String> {
    CACHE.with(|c| {
        let cache = c.borrow();
        if let Some((timestamp, value)) = cache.get(key) {
            let now = js_sys::Date::now();
            if now - timestamp < ttl_ms {
                return Some(value.clone());
            }
        }
        None
    })
}

pub fn set_cached(key: &str, value: &str) {
    CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        cache.insert(key.to_string(), (js_sys::Date::now(), value.to_string()));
    });
}

/// Typed read-through cache helper: returns a deserialized cache hit if one
/// is present and fresh, otherwise runs `fetch`, serializes the result into
/// the cache, and returns it. `None` is propagated from `fetch` unchanged
/// (and not cached).
pub async fn cached<T, F, Fut>(key: &str, ttl_ms: f64, fetch: F) -> Option<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    if let Some(hit) = get_cached(key, ttl_ms) {
        if let Ok(v) = serde_json::from_str(&hit) {
            return Some(v);
        }
    }
    let val = fetch().await?;
    if let Ok(json) = serde_json::to_string(&val) {
        set_cached(key, &json);
    }
    Some(val)
}
