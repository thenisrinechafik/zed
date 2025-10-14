#![cfg(all(feature = "win-git", target_os = "windows"))]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::debug;
use once_cell::sync::OnceCell;

const GCM_ENV_OVERRIDE: &str = "ZED_GCM_PATH";

pub fn configure_repository(repo: &git2::Repository) -> Result<()> {
    let mut config = repo.config().context("read repository config")?;

    if config.get_string("core.filemode").is_err() {
        config.set_bool("core.filemode", false).context("set core.filemode")?;
    }
    if config.get_string("core.autocrlf").is_err() {
        config
            .set_str("core.autocrlf", "false")
            .context("set core.autocrlf")?;
    }
    if config.get_string("core.eol").is_err() {
        config.set_str("core.eol", "lf").context("set core.eol")?;
    }

    Ok(())
}

pub fn credential_env_overrides() -> HashMap<String, String> {
    let mut env = HashMap::new();
    if let Some(path) = credential_helper_path() {
        env.insert("GIT_ASKPASS".into(), path.to_string_lossy().into_owned());
        env.insert("SSH_ASKPASS".into(), path.to_string_lossy().into_owned());
    }
    env
}

fn credential_helper_path() -> Option<PathBuf> {
    static HELPER: OnceCell<Option<PathBuf>> = OnceCell::new();
    HELPER
        .get_or_init(|| {
            if let Some(override_path) = std::env::var_os(GCM_ENV_OVERRIDE) {
                let path = PathBuf::from(override_path);
                if path.exists() {
                    debug!("windows git: using credential helper override at {:?}", path);
                    return Some(path);
                }
            }

            which::which("git-credential-manager").ok().or_else(|| which::which("git-credential-manager.exe").ok())
        })
        .clone()
}

pub fn normalize_path_for_git(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::from(path);
    if let Ok(stripped) = path.strip_prefix("\\\\?\\") {
        normalized = PathBuf::from(stripped);
    }
    normalized
}
