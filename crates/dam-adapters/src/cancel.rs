use std::sync::atomic::{AtomicBool, Ordering};

/// Set once by the process's interrupt handler and read by every loop that
/// waits on something slower than the operator's patience.
static REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_cancellation() {
    REQUESTED.store(true, Ordering::Relaxed);
}

pub fn cancellation_requested() -> bool {
    REQUESTED.load(Ordering::Relaxed)
}
