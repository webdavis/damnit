use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

#[derive(Clone, Debug, Default)]
pub struct Interrupt(Arc<AtomicBool>);

impl Interrupt {
    pub fn request(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn requested(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

static THE_HANDLE_THE_SIGNAL_HANDLER_SETS: OnceLock<Interrupt> = OnceLock::new();

extern "C" fn interrupt_handler_that_only_stores_the_flag(_: libc::c_int) {
    if let Some(interrupt) = THE_HANDLE_THE_SIGNAL_HANDLER_SETS.get() {
        interrupt.request();
    }
}

pub fn catch_interrupts(interrupt: &Interrupt) -> bool {
    if THE_HANDLE_THE_SIGNAL_HANDLER_SETS
        .set(interrupt.clone())
        .is_err()
    {
        return false;
    }
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
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn an_interrupt_after_the_handler_is_installed_sets_the_handle() {
        let interrupt = Interrupt::default();
        assert!(catch_interrupts(&interrupt));
        assert!(!catch_interrupts(&Interrupt::default()));
        let sent = std::process::Command::new("kill")
            .args(["-INT", &std::process::id().to_string()])
            .status()
            .unwrap();
        assert!(sent.success());
        let give_up_at = Instant::now() + Duration::from_secs(5);
        while !interrupt.requested() {
            assert!(Instant::now() < give_up_at, "the handle was never set");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
