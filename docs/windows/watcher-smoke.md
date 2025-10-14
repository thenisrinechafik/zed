# Windows file watcher smoke test

With the `fs` crate compiled using the `win-longpaths` feature the watcher accepts paths longer than 260 characters and preserves UTF-16 surrogate pairs emitted by `ReadDirectoryChangesW`.

## Quick stress loop

```powershell
pwsh scripts/win/build.ps1
pwsh -Command "Get-ChildItem test-data/watch-stress | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue"
pwsh -Command "New-Item -ItemType Directory test-data/watch-stress | Out-Null"
pwsh -Command "cargo test -p fs --features win-longpaths watcher_stress -- --ignored"
```

The test harness emits timing information and ensures no duplicate create/change sequences leak through the bounded channel used by the Windows watcher backend.
