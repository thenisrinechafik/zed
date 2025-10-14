#![cfg(all(feature = "win-perf", target_os = "windows"))]

use anyhow::Result;
use serde_json::{json, Value};
use std::time::Instant;

pub fn measure<F>(label: &str, mut action: F) -> Result<Value>
where
    F: FnMut() -> Result<()>,
{
    let start = Instant::now();
    action()?;
    let elapsed = start.elapsed().as_millis();
    Ok(json!({
        "metric": "open_latency_ms",
        "label": label,
        "value": elapsed,
    }))
}
