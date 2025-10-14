//! Crash reporting utilities shared by Windows release tooling.
use telemetry::Event;

#[cfg(all(target_os = "windows", feature = "win-crashops"))]
pub mod windows;

/// Initialize crash reporting for the current build.
#[cfg_attr(not(all(target_os = "windows", feature = "win-crashops")), allow(unused_variables))]
pub fn init_crash_reporting(build_id: &str) -> anyhow::Result<()> {
    #[cfg(all(target_os = "windows", feature = "win-crashops"))]
    {
        return windows::init_crash_reporting(build_id);
    }

    Ok(())
}

/// Record a privacy aware event.
pub fn record_opt_in(channel: &str) {
    telemetry::event!("Crash Reporting Opt In", channel = channel.to_string());
}

/// Helper to attach metadata to crash payloads.
pub fn crash_metadata(channel: &str, version: &str) -> Event {
    telemetry::Event {
        event_type: "Crash Metadata".to_string(),
        event_properties: std::collections::HashMap::from([
            ("channel".to_string(), serde_json::Value::String(channel.to_string())),
            ("version".to_string(), serde_json::Value::String(version.to_string())),
        ]),
    }
}
