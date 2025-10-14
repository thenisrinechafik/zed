# Windows Shell & Node bootstrap

This document captures the new shell adapters and managed Node.js flow introduced for the Windows shiproom.

## Shell adapters

When the `win-shell-adapters` feature is enabled we probe for additional shells beyond the stock PowerShell / Cmd pairing:

- Git Bash (MSYS2) – discovered via `which bash`
- Windows Subsystem for Linux – queried via `wsl.exe -l -q`

The helper exposed through `util::windows_shell_adapters` provides:

```rust
use std::time::Duration;
use util::windows_shell_adapters::{capture_env, detect_shells, ShellKind};

let shells = detect_shells();
let env_map = capture_env(ShellKind::GitBash, Duration::from_secs(10))?;
```

Environment output is normalised and merged with the host PATH before being forwarded to extensions.

## Managed Node runtime

`node_runtime::windows_support::ensure_node` now honours `HTTP_PROXY`, `HTTPS_PROXY`, and `NO_PROXY` when downloading the bundled Node.js archive. Successful downloads are copied to `%LOCALAPPDATA%\Zed\node\versions/<semver>` so repeated installs can operate offline.

```rust
use node_runtime::windows_support::{ensure_node, ProxySettings};

let proxy = ProxySettings::from_env();
let node = ensure_node("22.5.1", Some(&proxy))?;
println!("Node binary located at {}", node.display());
```

See `scripts/win/proxy-test.ps1` for a smoke test that exercises proxy and offline cache scenarios.
