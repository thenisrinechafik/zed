use std::env;

#[cfg(all(target_os = "windows", feature = "win-shell-adapters"))]
fn main() -> anyhow::Result<()> {
    let version = env::args().nth(1).unwrap_or_else(|| "22.5.1".to_string());
    let proxy = node_runtime::windows_support::ProxySettings::from_env();
    let path = node_runtime::windows_support::ensure_node(&version, Some(&proxy))?;
    println!("Node located at {}", path.display());
    Ok(())
}

#[cfg(not(all(target_os = "windows", feature = "win-shell-adapters")))]
fn main() {
    eprintln!("ensure_node example is Windows only");
}
