#![cfg(all(target_os = "windows", feature = "win-collab"))]

use collab::windows::proxy_from_environment;

#[test]
fn parses_proxy_from_environment() {
    std::env::set_var("HTTPS_PROXY", "http://127.0.0.1:8080");
    let proxy = proxy_from_environment().expect("proxy detection");
    assert!(proxy.is_some());
    std::env::remove_var("HTTPS_PROXY");
}

#[test]
fn ignores_invalid_proxy() {
    std::env::set_var("HTTPS_PROXY", "not-a-url");
    let proxy = proxy_from_environment().expect("proxy detection");
    assert!(proxy.is_none());
    std::env::remove_var("HTTPS_PROXY");
}
