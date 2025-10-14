# Zed on Windows — Bootstrap Guide

This document captures the minimum tooling required to produce a local Zed build on Windows ahead of the beta push.  The PowerShell scripts ship alongside these notes so the same steps can be exercised locally and in CI.

## Prerequisites

1. **Windows 11 or Windows 10 (22H2+)** with the latest cumulative updates.
2. **Visual Studio Build Tools 2022** with the following workloads:
   - *Desktop development with C++*
   - *Windows 11/10 SDK*
   - Optional: *C++ ATL/MFC* if you need to debug shell integration.
3. **CMake** (3.26 or newer) — installed via [kitware.com](https://cmake.org/download/) or `winget install Kitware.CMake`.
4. **Rustup** with the stable toolchain and the `x86_64-pc-windows-msvc` target.
5. Optional but recommended:
   - PostgreSQL 16+ for collaboration services (`script/postgres` remains unchanged).
   - `git` credential helpers (Git Credential Manager works out of the box).

> ℹ️ If you prefer automated provisioning, run `pwsh scripts/win/setup.ps1`. The script verifies toolchain availability, installs CMake via `winget` when missing, and enables static CRT linkage through the `ZED_STATIC_CRT` environment flag.

## Environment configuration

The workspace now exposes a `windows_port` feature group (see `Cargo.toml`) and honours the `ZED_STATIC_CRT` environment variable. When the variable is set, `.cargo/config.toml` adds `-C target-feature=+crt-static` so native dependencies such as LiveKit link correctly.

```
# Enable static CRT for the current shell
$Env:ZED_STATIC_CRT = '1'
```

The CI scripts set `ZED_WIN_BOOTSTRAP=1` to allow crates to detect that they are executing under the Windows bring-up flow.

## Build and test commands

```powershell
# Once per machine
pwsh scripts/win/setup.ps1 -StaticCRT

# Incremental checks and unit tests (excludes GPU-heavy crates by default)
pwsh scripts/win/build.ps1 -RunClippy -StaticCRT
```

To run the editor after building, launch:

```powershell
cargo run -p zed --features windows_port
```

The `windows_port` feature currently wires in manifest generation for the DirectX backend and is the natural home for subsequent Windows-only toggles.

## Corporate proxy notes

The build scripts respect standard `HTTP(S)_PROXY` variables. If you rely on an internal certificate authority, install it into the machine store so Rustup, Cargo and Node downloads succeed.

## Troubleshooting

- **Missing `cl.exe`** – rerun the Visual Studio installer and add the *Desktop development with C++* workload.
- **`cmake` not found** – install via `winget install Kitware.CMake` or add an existing installation to `%PATH%`.
- **LiveKit linking errors** – confirm `ZED_STATIC_CRT=1` is set before invoking Cargo.
- **Long path issues** – enable the `LongPathsEnabled` group policy or registry key (`HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`).

The Windows CI job publishes logs under the `target/win-ci-logs` directory for post-mortem analysis.
