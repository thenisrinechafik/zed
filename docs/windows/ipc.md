# Windows Single-Instance Diagnostics

The Windows build keeps a global mutex (`Global\\Zed_<uuid>`) and a named pipe
(`\\.\pipe\zed-launcher-<uuid>`) to ensure that secondary invocations forward
focus and open requests to the primary process.

## Environment variables

- `ZED_IPC_DEBUG=1` – enables verbose logging (`gpu.windows` category) for mutex
  acquisition, named-pipe connections, and payload forwarding.

## Manual smoke checklist

1. Launch Zed via the GUI and the CLI multiple times in quick succession – only
   a single window should remain open and subsequent invocations focus the
   original window within 200 ms.
2. Toggle `ZED_IPC_DEBUG=1` and verify that `logs/ipc/` under
   `%LOCALAPPDATA%\Zed` captures mutex ownership, stale-owner detection, and
   named-pipe connection attempts.
3. Run `cargo test -p platform --tests --features win-ipc` on a Windows host to
   stress the mutex and pipe coordination across 20 concurrent attempts.

## Troubleshooting

If a stale mutex owner prevents launch, remove `%LOCALAPPDATA%\Zed\logs\ipc` for
analysis and re-run with `ZED_IPC_DEBUG=1`. The helper automatically releases a
stale lock when the named pipe is missing.
