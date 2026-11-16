use super::*;

#[test]
fn an_interrupt_after_the_handler_is_installed_sets_the_handle() {
    let interrupt = Interrupt::default();
    assert!(catch_interrupts(&interrupt));
    assert!(!catch_interrupts(&Interrupt::default()));
    assert_eq!(unsafe { libc::raise(libc::SIGINT) }, 0);
    assert!(interrupt.requested());
}

#[test]
fn the_application_sees_an_interrupt_only_once_it_is_requested() {
    let interrupt = Interrupt::default();
    let seen_by_the_application: &dyn Interruption = &interrupt;
    assert!(!seen_by_the_application.requested());
    interrupt.request();
    assert!(seen_by_the_application.requested());
}
