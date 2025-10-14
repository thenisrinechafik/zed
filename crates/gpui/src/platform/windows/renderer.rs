use std::{env, fmt, path::PathBuf, time::SystemTime};

use super::WindowsVersion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HdrMode {
    Auto,
    On,
    Off,
}

impl HdrMode {
    fn from_env(value: Option<&str>) -> Self {
        match value.map(|v| v.trim().to_ascii_lowercase()) {
            Some(ref v) if v == "on" || v == "1" || v == "true" => HdrMode::On,
            Some(ref v) if v == "off" || v == "0" || v == "false" => HdrMode::Off,
            _ => HdrMode::Auto,
        }
    }

    pub(crate) fn should_enable(self, hdr_available: bool) -> bool {
        match self {
            HdrMode::Auto => hdr_available,
            HdrMode::On => true,
            HdrMode::Off => false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WindowsRendererConfig {
    vsync_enabled: bool,
    composition_enabled: bool,
    hdr: HdrMode,
    dred_enabled: bool,
    diagnostics_root: Option<PathBuf>,
}

impl WindowsRendererConfig {
    pub(crate) fn detect(version: WindowsVersion) -> Self {
        let vsync_enabled = parse_toggle(env::var("ZED_VSYNC").ok().as_deref(), true);
        let default_composition = cfg!(feature = "dx-composition") && matches!(version, WindowsVersion::Win11);
        let composition_enabled = parse_toggle(
            env::var("ZED_DX_COMPOSITION").ok().as_deref(),
            default_composition,
        );
        let hdr = HdrMode::from_env(env::var("ZED_HDR").ok().as_deref());
        let dred_enabled = parse_toggle(env::var("ZED_DX_DRED").ok().as_deref(), true)
            && cfg!(feature = "dx-dred");
        let diagnostics_root = diagnostics_directory();

        Self {
            vsync_enabled,
            composition_enabled,
            hdr,
            dred_enabled,
            diagnostics_root,
        }
    }

    pub(crate) fn vsync_enabled(&self) -> bool {
        self.vsync_enabled
    }

    pub(crate) fn composition_enabled(&self, disable_direct_composition: bool) -> bool {
        self.composition_enabled && !disable_direct_composition
    }

    pub(crate) fn hdr_mode(&self) -> HdrMode {
        self.hdr
    }

    #[cfg(feature = "dx-dred")]
    pub(crate) fn dred_enabled(&self) -> bool {
        self.dred_enabled
    }

    pub(crate) fn diagnostics_root(&self) -> Option<PathBuf> {
        self.diagnostics_root.clone()
    }

    pub(crate) fn ensure_diagnostics_root(&self) -> Option<PathBuf> {
        self.diagnostics_root.as_ref().map(|path| {
            let _ = std::fs::create_dir_all(path);
            path.clone()
        })
    }

    #[cfg(feature = "dx-dred")]
    pub(crate) fn dred_dump_path(&self) -> Option<PathBuf> {
        let root = self.ensure_diagnostics_root()?;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        Some(root.join(format!("dred-{timestamp}.txt")))
    }
}

pub(crate) fn clamp_swapchain_extent(width: u32, height: u32) -> (u32, u32) {
    (width.max(1), height.max(1))
}

pub(crate) fn dpi_to_physical(width: f32, height: f32, dpi: f32) -> (u32, u32) {
    let scale = (dpi / 96.0).max(0.5);
    let w = (width * scale).round().max(1.0);
    let h = (height * scale).round().max(1.0);
    (w as u32, h as u32)
}

pub(crate) fn parse_toggle(value: Option<&str>, default: bool) -> bool {
    match value {
        Some(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("on")
        }
        None => default,
    }
}

fn diagnostics_directory() -> Option<PathBuf> {
    match env::var_os("LOCALAPPDATA") {
        Some(root) => {
            let mut path = PathBuf::from(root);
            path.push("Zed");
            path.push("logs");
            path.push("gpu");
            Some(path)
        }
        None => None,
    }
}

impl fmt::Display for HdrMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HdrMode::Auto => write!(f, "auto"),
            HdrMode::On => write!(f, "on"),
            HdrMode::Off => write!(f, "off"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toggle_values() {
        assert!(parse_toggle(Some("1"), false));
        assert!(parse_toggle(Some("true"), false));
        assert!(parse_toggle(Some("True"), false));
        assert!(!parse_toggle(Some("0"), true));
        assert!(!parse_toggle(Some("off"), true));
        assert!(parse_toggle(None, true));
        assert!(!parse_toggle(None, false));
    }

    #[test]
    fn computes_clamped_extent() {
        assert_eq!(clamp_swapchain_extent(0, 0), (1, 1));
        assert_eq!(clamp_swapchain_extent(1920, 1080), (1920, 1080));
    }

    #[test]
    fn converts_dpi_to_physical() {
        let (width, height) = dpi_to_physical(100.0, 50.0, 192.0);
        assert_eq!((width, height), (200, 100));
        let (width, height) = dpi_to_physical(1.0, 1.0, 24.0);
        assert_eq!((width, height), (1, 1));
    }

    #[test]
    fn hdr_mode_from_env() {
        assert_eq!(HdrMode::from_env(Some("on")), HdrMode::On);
        assert_eq!(HdrMode::from_env(Some("Off")), HdrMode::Off);
        assert_eq!(HdrMode::from_env(None), HdrMode::Auto);
    }
}
