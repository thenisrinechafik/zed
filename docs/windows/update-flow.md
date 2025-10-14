# Windows Update Flow (Release Candidate)

The Windows updater uses staged delta packages with automatic rollback. The helper writes structured logs into `%LOCALAPPDATA%\Zed\logs\updater`.

## Test Matrix

1. Delta update success
2. Delta update fallback to full package
3. Rollback after simulated failure (`ZED_UPDATE_FAIL=1`)
4. Channel migration (Preview → Stable)

Collect logs with `scripts/win/collect-logs.ps1` when filing issues.
