#![cfg(all(target_os = "windows", feature = "win-shell-adapters"))]

use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use log::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellKind {
    Powershell,
    Cmd,
    Nushell,
    GitBash,
    Wsl,
}

pub fn detect_shells() -> Vec<ShellKind> {
    let mut shells = vec![ShellKind::Powershell, ShellKind::Cmd, ShellKind::Nushell];
    if gitbash_command().is_some() {
        shells.push(ShellKind::GitBash);
    }
    if wsl_default_distro().is_some() {
        shells.push(ShellKind::Wsl);
    }
    shells
}

pub fn capture_env(shell: ShellKind, timeout: Duration) -> Result<HashMap<String, String>> {
    let _ = timeout;
    let output = match shell {
        ShellKind::Powershell => run_command("powershell", &["-NoProfile", "-Command", "Get-ChildItem Env:"])?
            .stdout,
        ShellKind::Cmd => run_command("cmd", &["/C", "set"])?
            .stdout,
        ShellKind::Nushell => run_command("nu", &["-c", "print $env"])?
            .stdout,
        ShellKind::GitBash => {
            let cmd = gitbash_command().context("Git Bash not found")?;
            run_command(&cmd, &["--login", "-i", "-c", "env"])?
                .stdout
        }
        ShellKind::Wsl => {
            let distro = wsl_default_distro().context("WSL not available")?;
            run_command("wsl", &["-d", &distro, "env"])?
                .stdout
        }
    };

    parse_env_output(&output)
}

fn parse_env_output(bytes: &[u8]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for line in bytes.split(|b| *b == b'\n') {
        if let Some((key, value)) = line.split_once(|b| *b == b'=') {
            let key = String::from_utf8_lossy(key).trim().to_string();
            let value = String::from_utf8_lossy(value).trim().to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    Ok(map)
}

fn run_command(program: &str, args: &[&str]) -> Result<std::process::Output> {
    debug!("capturing environment from {program:?} with args {args:?}");
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("launching {program}"))?;
    anyhow::ensure!(
        output.status.success(),
        "command {program} failed: {:?}",
        output.status
    );
    Ok(output)
}

fn gitbash_command() -> Option<String> {
    which::which("bash")
        .ok()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}

fn wsl_default_distro() -> Option<String> {
    Command::new("wsl")
        .args(["-l", "-q"])
        .output()
        .ok()
        .and_then(|output| {
            if !output.status.success() {
                return None;
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().next().map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}
