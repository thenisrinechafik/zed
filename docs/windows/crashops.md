# Crash Reporting & Privacy Controls

Crash reporting is optional. Users are prompted after the first crash and can manage telemetry under **Settings → Privacy**.

## Symbol Uploads

Run `gh workflow run symbols.yml` after tagging a release to publish PDB symbols.

## Redaction

The `crashops` crate removes file paths outside the workspace and hashes user identifiers before upload.
