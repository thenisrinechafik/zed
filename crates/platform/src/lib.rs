#![allow(clippy::result_large_err)]

#[cfg(all(feature = "win-ipc", target_os = "windows"))]
pub mod windows;

#[cfg(not(all(feature = "win-ipc", target_os = "windows")))]
pub mod windows {
    /// Stub definitions for non-Windows targets when the `win-ipc` feature is disabled.
    pub mod single_instance {
        use anyhow::{anyhow, Result};

        #[derive(Debug, Clone, Copy)]
        pub struct InstanceGuard;

        impl InstanceGuard {
            pub fn is_primary(&self) -> bool {
                false
            }
        }

        pub fn acquire_lock() -> Result<InstanceGuard> {
            Err(anyhow!("single-instance coordination is only available on Windows"))
        }

        pub fn signal_running_instance(_payload: &[u8]) -> Result<()> {
            Err(anyhow!("single-instance coordination is only available on Windows"))
        }
    }
}
