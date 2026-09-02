use std::sync::Mutex;
use std::thread::JoinHandle;

/// Manages background threads spawned by the TUI event loop.
///
/// Tracks all active `JoinHandle`s so they can be joined on shutdown,
/// preventing dangling threads from writing to closed channels.
pub struct TaskManager {
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Spawn a new background task and track its handle.
    /// Thread-spawn or lock failures are logged and never abort the app.
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match std::thread::Builder::new()
            .name("gitnapse-worker".into())
            .spawn(f)
        {
            Ok(handle) => {
                if let Ok(mut handles) = self.handles.lock() {
                    handles.push(handle);
                }
            }
            Err(e) => log::error!("failed to spawn background thread: {e}"),
        }
    }

    /// Remove handles for threads that have already completed.
    pub fn cleanup(&self) {
        let mut handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        handles.retain(|h| !h.is_finished());
    }

    /// Number of currently tracked (running) threads.
    pub fn active_count(&self) -> usize {
        let handles = self.handles.lock().unwrap_or_else(|e| e.into_inner());
        handles.len()
    }

    /// Join (wait for) all tracked threads to finish.
    /// Called during shutdown to ensure clean teardown.
    pub fn join_all(&self) {
        let mut handles =
            std::mem::take(&mut *self.handles.lock().unwrap_or_else(|e| e.into_inner()));
        for h in handles.drain(..) {
            let _ = h.join();
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
