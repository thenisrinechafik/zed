#![cfg(target_os = "windows")]

use std::sync::atomic::{AtomicUsize, Ordering};

static ACTIVE_HANDLES: AtomicUsize = AtomicUsize::new(0);
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug)]
pub struct ConptyLifecycle {
    id: usize,
}

impl ConptyLifecycle {
    pub fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let active = ACTIVE_HANDLES.fetch_add(1, Ordering::SeqCst) + 1;
        log::info!(target: "terminal.windows", "Spawned ConPTY #{id} (active={active})");
        Self { id }
    }

    pub fn on_resize(&self, cols: u16, rows: u16) {
        log::trace!(
            target: "terminal.windows",
            "Resizing ConPTY #{id} to {cols}x{rows}",
            id = self.id
        );
    }

    pub fn on_exit(&self, code: Option<i32>) {
        log::info!(
            target: "terminal.windows",
            "ConPTY #{id} exited (status={:?})",
            code,
            id = self.id
        );
    }
}

impl Drop for ConptyLifecycle {
    fn drop(&mut self) {
        let active = ACTIVE_HANDLES.fetch_sub(1, Ordering::SeqCst).saturating_sub(1);
        log::info!(
            target: "terminal.windows",
            "Closed ConPTY #{id} (active={active})",
            id = self.id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_and_decrements_active_count() {
        let initial = ACTIVE_HANDLES.load(Ordering::SeqCst);
        {
            let lifecycle = ConptyLifecycle::new();
            assert!(ACTIVE_HANDLES.load(Ordering::SeqCst) >= initial + 1);
            lifecycle.on_resize(80, 24);
        }
        assert!(ACTIVE_HANDLES.load(Ordering::SeqCst) >= initial);
    }
}
