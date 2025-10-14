use anyhow::Result;
use log::info;

pub fn init_crash_reporting(build_id: &str) -> Result<()> {
    info!("Initializing Windows crash reporting for build {build_id}");
    Ok(())
}
