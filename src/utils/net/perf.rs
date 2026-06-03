use std::cell::RefCell;

/// Cap on retained latency samples so the rolling average does not
/// drift unboundedly while the SPA is open.
const MAX_SAMPLES: usize = 200;

thread_local! {
    static LATENCY_SAMPLES: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

/// Record an API call metric. Only `duration_ms` is retained; the
/// other fetch metadata used to be stored too but no surface consumed
/// it.
pub fn record_api_call(_url: &str, _method: &str, duration_ms: f64, _status: u16) {
    LATENCY_SAMPLES.with(|m| {
        let mut samples = m.borrow_mut();
        samples.push(duration_ms);
        if samples.len() > MAX_SAMPLES {
            let drain_count = samples.len() - MAX_SAMPLES;
            samples.drain(..drain_count);
        }
    });
}

/// Return the average latency (in ms) across all recorded metrics, or 0.0 if none.
pub fn average_latency() -> f64 {
    LATENCY_SAMPLES.with(|m| {
        let samples = m.borrow();
        if samples.is_empty() {
            return 0.0;
        }
        let total: f64 = samples.iter().sum();
        total / samples.len() as f64
    })
}
