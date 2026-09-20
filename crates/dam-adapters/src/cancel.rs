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

/// The whole interrupt handler: it stores the cancellation flag and returns,
/// which is all a signal handler may safely do.
extern "C" fn on_interrupt(_: libc::c_int) {
    request_cancellation();
}

/// Routes SIGINT into the cancellation flag every wait loop reads. `false`
/// when the handler could not be installed, which leaves Ctrl-C at its default
/// behaviour rather than cancelling cleanly.
pub fn catch_interrupts() -> bool {
    // SAFETY: `on_interrupt` only stores into a static atomic, which is
    // async-signal-safe, and nothing here reads the previous handler.
    let installed = unsafe {
        libc::signal(
            libc::SIGINT,
            on_interrupt as *const () as libc::sighandler_t,
        )
    };
    installed != libc::SIG_ERR
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs in the test process, so it must leave the flag as it found it.
    #[test]
    fn an_interrupt_after_the_handler_is_installed_sets_the_flag() {
        assert!(catch_interrupts());
        // SAFETY: raising SIGINT in this process runs the handler above,
        // which only stores into a static atomic.
        unsafe { libc::raise(libc::SIGINT) };
        assert!(cancellation_requested());
        REQUESTED.store(false, Ordering::Relaxed);
    }
}
