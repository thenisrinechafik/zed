# Windows Release Workflow

This document summarizes the packaging and signing steps used for the Windows public beta release candidate.

## Build Matrix

| Channel | Feature Flags | Installer | Notes |
|---------|---------------|-----------|-------|
| Stable  | `windows_port,win-release,win-updates-final` | Inno + MSIX | Default channel |
| Preview | `windows_port,win-release` | Inno + MSIX | Co-installs with Stable |
| Nightly | `windows_port,win-release` | Inno only | No automatic updates |

## Commands

```
pwsh scripts/win/package.ps1 -Channel Stable -Version 1.0.0-beta1
pwsh scripts/win/sign.ps1 -Artifacts dist/windows/out/Stable
pwsh scripts/win/verify.ps1 -Artifacts dist/windows/out/Stable
```

Signed artifacts are uploaded by `.github/workflows/release-windows.yml`.
