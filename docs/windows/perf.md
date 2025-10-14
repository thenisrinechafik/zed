# Windows Performance Gates

Performance benchmarks live in `bench/windows` and are enabled with the
`win-perf` feature. Results are emitted as JSON fragments that downstream CI can
aggregate.

## Benchmarks

- `open_latency::measure` – times project opening flows.
- `watcher_throughput::record` – records file-watcher events per second.

Invoke the helpers from integration scripts and append the emitted JSON to
`bench/windows/results.json` for ingestion by dashboards.

## Commands

```powershell
cargo test -p bench-windows --features win-perf
```

The tests do not execute the benchmarks but ensure they compile against the
Windows-only runtime.

## Manual checklist

1. Capture an `open_latency` sample while launching a 100k-file workspace.
2. Record `watcher_throughput` while editing a churn-heavy repository.
3. Attach the JSON artefacts to the nightly perf run.
