use dam_application::{HelperError, RemoteHelper};

use super::super::super::testing::{install_listening_to, remote};
use crate::Interrupt;

#[test]
fn an_interrupt_on_its_own_handle_cancels_a_silent_helper_and_reaps_it() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let launcher = install_listening_to(
        dir.path(),
        "#!/bin/sh\nwhile :; do sleep 0.05; done\n",
        &interrupt,
    );
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    let pid = helper.child.id().to_string();
    interrupt.request();
    assert_eq!(helper.capabilities().unwrap_err(), HelperError::Cancelled);
    assert!(
        !std::process::Command::new("kill")
            .args(["-0", &pid])
            .output()
            .unwrap()
            .status
            .success(),
        "the cancelled helper was left running or unreaped"
    );
}
