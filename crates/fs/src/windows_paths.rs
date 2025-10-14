#![cfg(all(target_os = "windows", feature = "win-longpaths"))]

use std::char::decode_utf16;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn to_win_long_path(path: &Path) -> PathBuf {
    if !path.is_absolute() {
        return path.to_path_buf();
    }

    let path_str = path.as_os_str().to_string_lossy();
    if path_str.starts_with(r"\\\\?\") {
        path.to_path_buf()
    } else {
        PathBuf::from(format!(r"\\\\?\{}", path_str))
    }
}

pub fn normalize_utf16(units: &[u16]) -> Result<String> {
    let decoded = decode_utf16(units.iter().copied())
        .map(|r| r.map_err(|e| anyhow::anyhow!("invalid UTF-16 sequence: {e}")))
        .collect::<Result<String>>()?;
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_path_prefix() {
        let path = PathBuf::from(r"C:\\Temp");
        let long = to_win_long_path(&path);
        assert!(long.to_string_lossy().starts_with(r"\\\\?\"));
    }

    #[test]
    fn test_normalize_utf16_basic() {
        let units = [0x0041u16, 0xD83D, 0xDE00];
        let text = normalize_utf16(&units).unwrap();
        assert_eq!(text, "A😀");
    }
}
