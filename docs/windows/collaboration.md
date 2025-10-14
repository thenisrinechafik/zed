# Collaboration on Windows

The Windows collaboration stack configures `ReqwestClient` with platform TLS
(`http_client_tls`) and honours `HTTP(S)_PROXY` and `NO_PROXY` when connecting to
LiveKit, AWS, or Zed services.

## Proxy-aware HTTP client

`collab::windows::http_client_for_user_agent` wraps `reqwest` so that
corporate proxies are used automatically. Tests under
`crates/collab/tests/collab_tls.rs` exercise the proxy detector.

## Manual smoke checklist

1. Set `HTTPS_PROXY` to a local MITM proxy and launch the collaboration server;
   verify that API calls succeed and the proxy observes TLS handshakes.
2. Join a LiveKit session and confirm audio/video devices are detected via
   `media::windows_audio::enumerate_devices`.
3. Run `cargo test -p collab --tests --features win-collab` on a Windows host.

## Troubleshooting

- Use `RUST_LOG=info` to inspect proxy selection and TLS bootstrap.
- Override device enumeration by logging the output of
  `media::windows_audio::enumerate_devices()`.
