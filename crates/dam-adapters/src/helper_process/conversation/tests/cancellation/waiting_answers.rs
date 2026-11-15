use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use dam_application::{
    HelperError, MutationOp, MutationOutcome, MutationVerdict, PushOutcome, RemoteHelper,
    RemoteMutation,
};
use dam_domain::Oid;

use super::super::super::super::testing::{install_listening_to, remote};
use crate::Interrupt;

const ANSWER_A: &str = r#"{"early_answer":{"oid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","ok":true,"remote_id":"ra"}}"#;
const ANSWER_B: &str = r#"{"early_answer":{"oid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","ok":true,"remote_id":"rb"}}"#;
const FINAL_REPLY: &str = r#"{"results":[{"oid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","ok":true,"remote_id":"ra"},{"oid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","ok":true,"remote_id":"rb"}]}"#;
const PULL_REPLY: &str = r#"{"objects":[],"removed":[],"sync":null}"#;

fn marker(dir: &Path) -> PathBuf {
    dir.join("written")
}

fn printing(line: &str) -> String {
    format!("printf '{line}\\n'\n")
}

fn marking_then_silent(dir: &Path, steps: &str) -> String {
    let marker = marker(dir).display().to_string();
    format!("#!/bin/sh\n{steps}: > '{marker}'\nexec sleep 1000\n")
}

fn until_the_helper_has_written(dir: &Path) {
    let give_up_at = Instant::now() + Duration::from_secs(10);
    while !marker(dir).exists() {
        assert!(Instant::now() < give_up_at, "the helper wrote nothing");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn delete(c: char) -> RemoteMutation {
    RemoteMutation {
        op: MutationOp::Delete,
        oid: Oid::parse(&c.to_string().repeat(40)).unwrap(),
        idempotency_key: format!("k{c}"),
        remote_id: Some(format!("r{c}")),
        container_remote_id: None,
        object: None,
        fields: vec![],
    }
}

fn accepted(c: char) -> MutationOutcome {
    MutationOutcome {
        oid: Oid::parse(&c.to_string().repeat(40)).unwrap(),
        verdict: MutationVerdict::Accepted {
            remote_id: Some(format!("r{c}")),
        },
    }
}

fn pushed_after_an_interrupt(lines: &[&str]) -> PushOutcome {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let steps: String = lines.iter().map(|l| printing(l)).collect();
    let script = marking_then_silent(dir.path(), &steps);
    let launcher = install_listening_to(dir.path(), &script, &interrupt);
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    until_the_helper_has_written(dir.path());
    interrupt.request();
    helper.push(vec![delete('a'), delete('b')]).unwrap()
}

#[test]
fn an_interrupt_keeps_the_answers_already_waiting() {
    assert_eq!(
        pushed_after_an_interrupt(&[ANSWER_A, ANSWER_B]),
        PushOutcome {
            outcomes: vec![accepted('a'), accepted('b')],
            stopped_by: Some(HelperError::Cancelled),
        }
    );
}

#[test]
fn an_interrupt_after_the_final_reply_arrived_keeps_it_all_and_still_cancels() {
    assert_eq!(
        pushed_after_an_interrupt(&[FINAL_REPLY]),
        PushOutcome {
            outcomes: vec![accepted('a'), accepted('b')],
            stopped_by: Some(HelperError::Cancelled),
        }
    );
}

#[test]
fn an_interrupt_ignores_a_pull_reply_already_waiting() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let script = marking_then_silent(dir.path(), &printing(PULL_REPLY));
    let launcher = install_listening_to(dir.path(), &script, &interrupt);
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    until_the_helper_has_written(dir.path());
    interrupt.request();
    assert_eq!(helper.pull(None).unwrap_err(), HelperError::Cancelled);
}

#[test]
fn an_interrupt_while_dam_waits_keeps_the_answer_that_came_before_it() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let steps = format!("read -r line\n{}", printing(ANSWER_A));
    let script = marking_then_silent(dir.path(), &steps);
    let launcher = install_listening_to(dir.path(), &script, &interrupt);
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    let watched = dir.path().to_path_buf();
    let pressing = interrupt.clone();
    let presser = std::thread::spawn(move || {
        until_the_helper_has_written(&watched);
        pressing.request();
    });
    let pushed = helper.push(vec![delete('a')]).unwrap();
    presser.join().unwrap();
    assert_eq!(
        pushed,
        PushOutcome {
            outcomes: vec![accepted('a')],
            stopped_by: Some(HelperError::Cancelled),
        }
    );
}

fn a_child_that_outlives_the_helper_writing_every_20ms(dir: &Path) -> String {
    let marker = marker(dir).display().to_string();
    format!(
        "#!/bin/sh\n(n=0; while [ $n -lt 200 ]; do printf '{ANSWER_A}\\n'; : > '{marker}'; sleep 0.02; n=$((n + 1)); done) &\nwait\n"
    )
}

#[test]
fn answers_that_keep_coming_after_the_interrupt_stop_being_read_within_one_grace() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let script = a_child_that_outlives_the_helper_writing_every_20ms(dir.path());
    let launcher = install_listening_to(dir.path(), &script, &interrupt);
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    until_the_helper_has_written(dir.path());
    interrupt.request();
    let started = Instant::now();
    let pushed = helper.push(vec![delete('a')]).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "dam kept reading for {:?} after the interrupt",
        started.elapsed()
    );
    assert_eq!(pushed.stopped_by, Some(HelperError::Cancelled));
}
