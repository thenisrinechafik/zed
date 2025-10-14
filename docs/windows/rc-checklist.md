# Windows Release Candidate Checklist

- [ ] Run `pwsh qa/windows/sanity.ps1`
- [ ] Execute `pwsh qa/windows/perf.ps1`
- [ ] Validate media capture with `pwsh qa/windows/media.ps1`
- [ ] Verify git workflows via `pwsh qa/windows/git.ps1`
- [ ] Confirm update flow using `tests/updater_rc.rs`
- [ ] Collect logs bundle and attach to QA notes
