#![cfg(all(feature = "win-collab-audio", target_os = "windows"))]

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};

pub struct WindowsAudioInventory {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

pub fn enumerate_devices() -> Result<WindowsAudioInventory> {
    let host = cpal::default_host();
    let inputs = host
        .input_devices()
        .context("enumerate input devices")?
        .filter_map(|device| device.name().ok())
        .collect::<Vec<_>>();
    let outputs = host
        .output_devices()
        .context("enumerate output devices")?
        .filter_map(|device| device.name().ok())
        .collect::<Vec<_>>();
    Ok(WindowsAudioInventory { inputs, outputs })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_returns_without_error() {
        let _ = enumerate_devices();
    }
}
