#![cfg(all(target_os = "windows", feature = "win-ipc"))]

use std::sync::Arc;
use std::thread;

use platform::windows::single_instance::acquire_lock;

#[test]
fn only_one_primary_instance_is_created() {
    let primary = acquire_lock().expect("failed to acquire primary lock");
    assert!(primary.is_primary());
    let attempts = 20;
    let secondary_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..attempts {
        let secondary_count = secondary_count.clone();
        handles.push(thread::spawn(move || {
            let guard = acquire_lock().expect("acquire lock");
            if guard.is_primary() {
                panic!("secondary attempt unexpectedly became primary");
            }
            secondary_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            drop(guard);
            // Exercise the named pipe round-trip for coverage.
            platform::windows::single_instance::signal_running_instance(b"ping").ok();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(secondary_count.load(std::sync::atomic::Ordering::SeqCst), attempts);
    drop(primary);
}
