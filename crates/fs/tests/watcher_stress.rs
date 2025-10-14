#![cfg(all(target_os = "windows", feature = "win-longpaths"))]

use fs::fs_watcher::FsWatcher;
use smol::channel;
use std::sync::Arc;
use parking_lot::Mutex;

#[test]
fn smoke_register_long_path() {
    let (tx, _rx) = channel::unbounded();
    let pending = Arc::new(Mutex::new(Vec::new()));
    let watcher = FsWatcher::new(tx, pending);
    let base = std::env::temp_dir().join("zed-long-path-test");
    std::fs::create_dir_all(&base).unwrap();
    let long = base.join("a".repeat(260));
    std::fs::create_dir_all(&long).unwrap();
    watcher.add(&long).expect("watch registration");
}
