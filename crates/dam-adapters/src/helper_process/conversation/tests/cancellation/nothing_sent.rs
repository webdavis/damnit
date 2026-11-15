use std::path::Path;
use std::time::{Duration, Instant};

use dam_application::{HelperError, PushOutcome, RemoteHelper};

use super::super::super::super::testing::{install_listening_to, remote};
use crate::Interrupt;

fn until_it_exists(file: &Path) {
    let give_up_at = Instant::now() + Duration::from_secs(10);
    while !file.exists() {
        assert!(
            Instant::now() < give_up_at,
            "{} never appeared",
            file.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn a_helper_whose_child_keeps_every_request(dir: &Path) -> String {
    let dir = dir.display();
    format!(
        "#!/bin/sh\nexec 3<&0\n\
         (cat <&3 > '{dir}/requests'; : > '{dir}/input-closed') >/dev/null 2>&1 &\n\
         : > '{dir}/started'\nexec sleep 1000\n"
    )
}

#[test]
fn a_push_after_an_interrupt_sends_the_helper_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let script = a_helper_whose_child_keeps_every_request(dir.path());
    let launcher = install_listening_to(dir.path(), &script, &interrupt);
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    until_it_exists(&dir.path().join("started"));
    interrupt.request();

    let pushed = helper.push(vec![]).unwrap();
    drop(helper);

    assert_eq!(
        pushed,
        PushOutcome {
            outcomes: vec![],
            stopped_by: Some(HelperError::Cancelled),
        }
    );
    until_it_exists(&dir.path().join("input-closed"));
    assert_eq!(std::fs::read(dir.path().join("requests")).unwrap(), b"");
}

#[test]
fn a_pull_after_an_interrupt_sends_the_helper_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let script = a_helper_whose_child_keeps_every_request(dir.path());
    let launcher = install_listening_to(dir.path(), &script, &interrupt);
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    until_it_exists(&dir.path().join("started"));
    interrupt.request();

    let pulled = helper.pull(None);
    drop(helper);

    assert_eq!(pulled.unwrap_err(), HelperError::Cancelled);
    until_it_exists(&dir.path().join("input-closed"));
    assert_eq!(std::fs::read(dir.path().join("requests")).unwrap(), b"");
}
