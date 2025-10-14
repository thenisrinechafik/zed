#![cfg(all(target_os = "windows", feature = "win-git"))]

use git::windows::{credential_env_overrides, normalize_path_for_git};
use std::path::PathBuf;

#[test]
fn credential_override_uses_env_var() {
    std::env::set_var("ZED_GCM_PATH", r"C:\\temp\\gcm.exe");
    let env = credential_env_overrides();
    assert_eq!(
        env.get("GIT_ASKPASS").cloned(),
        Some(String::from(r"C:\\temp\\gcm.exe"))
    );
    std::env::remove_var("ZED_GCM_PATH");
}

#[test]
fn normalize_path_removes_extended_prefix() {
    let input = PathBuf::from(r"\\\\?\\C:\\workspace\\repo");
    let normalized = normalize_path_for_git(&input);
    assert_eq!(normalized, PathBuf::from(r"C:\\workspace\\repo"));
}
