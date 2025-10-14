# Git on Windows

Zed integrates with Git Credential Manager (GCM) when available and falls back
on the named-pipe askpass transport. The helper automatically honours proxy
settings and normalises long-paths so NTFS repositories behave consistently.

## Credential helper detection

- Override detection with `ZED_GCM_PATH` to point at a specific helper binary.
- When unset, Zed probes `git-credential-manager(.exe)` on `PATH`. If GCM is not
  found, askpass is used.

## Repository defaults

The Windows-specific bootstrap clears POSIX-only expectations:

- `core.filemode = false`
- `core.autocrlf = false`
- `core.eol = lf`

These defaults are only applied when the repository has not already specified a
value.

## Manual smoke checklist

1. In a mixed-CRLF repository, confirm that diffs no longer report spurious
   filemode changes and that CRLF/LF conversions are stable.
2. With `ZED_GCM_PATH` pointing to a test helper, trigger a remote fetch and
   inspect `%LOCALAPPDATA%\Zed\logs\git` for the detected credential strategy.
3. Run `cargo test -p git --tests --features win-git` on a Windows host.
