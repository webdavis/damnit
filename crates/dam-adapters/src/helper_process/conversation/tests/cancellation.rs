use std::time::Duration;

use dam_application::{HelperError, MutationOp, RemoteHelper, RemoteMutation};
use dam_domain::Oid;

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

const ANSWERS_FASTER_THAN_THE_ADAPTER_RECHECKS_THEN_REPLIES: &str = "#!/bin/sh
read -r line
n=0
while [ $n -lt 300 ]; do
  printf '{\"early_answer\":{\"oid\":\"$OID\",\"ok\":true,\"remote_id\":\"r1\"}}\\n'
  sleep 0.005
  n=$((n + 1))
done
printf '{\"results\":[{\"oid\":\"$OID\",\"ok\":true,\"remote_id\":\"r1\"}]}\\n'
";

#[test]
fn an_interrupt_is_noticed_while_answers_keep_coming() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let launcher = install_listening_to(
        dir.path(),
        ANSWERS_FASTER_THAN_THE_ADAPTER_RECHECKS_THEN_REPLIES,
        &interrupt,
    );
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    let pressing = interrupt.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        pressing.request();
    });
    let pushed = helper
        .push(vec![RemoteMutation {
            op: MutationOp::Delete,
            oid: Oid::parse(&"1".repeat(40)).unwrap(),
            idempotency_key: "k1".into(),
            remote_id: Some("r1".into()),
            container_remote_id: None,
            object: None,
            fields: vec![],
        }])
        .unwrap();
    assert_eq!(pushed.stopped_by, Some(HelperError::Cancelled));
}
