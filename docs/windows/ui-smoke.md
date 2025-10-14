# Windows UI Smoke Checklist

Use the `win_smoke` example to validate basic rendering and input plumbing on Windows builds.

## Prerequisites

1. Build the example:
   ```powershell
   pwsh scripts\win\build.ps1 -SkipTests
   cargo build -p gpui --example win_smoke --features win-smoke
   ```
2. Ensure the DirectX diagnostic directory exists at `%LOCALAPPDATA%\Zed\logs\gpu`.

## Test Matrix

| Scenario | Steps | Expected Result |
| --- | --- | --- |
| Swap chain resize | Launch `cargo run -p gpui --example win_smoke --features win-smoke`, resize window repeatedly including snapping to screen edges. | No device-lost logs, background color stretches cleanly. |
| HDR toggle | With an HDR-capable monitor, toggle Windows HDR setting while the window is visible. | `gpu.windows` log reports HDR enable/disable, window remains responsive. |
| VSync toggle | Run with `ZED_VSYNC=0` and `ZED_VSYNC=1`. | FPS counter reflects uncapped / capped presentation respectively. |
| Input logging | Type keys and move mouse over window. | `win_smoke` logger shows key and mouse traces; no duplicated keypresses. |
| Alt+F4 exit | Press Alt+F4 or close button. | Process exits cleanly with log line `Closed ConPTY` absent (example has no PTY). |

## Log Collection

If GPU diagnostics are required, gather `%LOCALAPPDATA%\Zed\logs\gpu` contents and attach to the issue or CI artifact.
