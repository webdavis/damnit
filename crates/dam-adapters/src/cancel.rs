use std::sync::atomic::{AtomicBool, Ordering};

static CANCELLATION_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_cancellation() {
    CANCELLATION_REQUESTED.store(true, Ordering::Relaxed);
}

pub fn cancellation_requested() -> bool {
    CANCELLATION_REQUESTED.load(Ordering::Relaxed)
}

extern "C" fn interrupt_handler_that_only_stores_the_flag(_: libc::c_int) {
    request_cancellation();
}

pub fn catch_interrupts() -> bool {
    // SAFETY: the handler only flips one shared on/off switch, which is safe to do even in the
    // middle of other code, and the handler this replaces is never looked at.
    let installed = unsafe {
        libc::signal(
            libc::SIGINT,
            interrupt_handler_that_only_stores_the_flag as *const () as libc::sighandler_t,
        )
    };
    installed != libc::SIG_ERR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_interrupt_after_the_handler_is_installed_sets_the_flag() {
        assert!(catch_interrupts());
        // SAFETY: raising the interrupt here only runs the handler above, and all it does is
        // flip one shared on/off switch.
        unsafe { libc::raise(libc::SIGINT) };
        assert!(cancellation_requested());
        leave_the_flag_as_the_rest_of_the_test_process_found_it();
    }

    fn leave_the_flag_as_the_rest_of_the_test_process_found_it() {
        CANCELLATION_REQUESTED.store(false, Ordering::Relaxed);
    }
}
