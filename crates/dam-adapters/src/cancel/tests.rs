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
