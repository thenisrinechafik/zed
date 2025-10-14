#![cfg(all(feature = "win-perf", target_os = "windows"))]

use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;

pub fn record(events_processed: usize, interval: Duration) -> Result<Value> {
    let throughput = if interval.is_zero() {
        0.0
    } else {
        events_processed as f64 / interval.as_secs_f64()
    };
    Ok(json!({
        "metric": "watcher_events_per_sec",
        "value": throughput,
    }))
}
