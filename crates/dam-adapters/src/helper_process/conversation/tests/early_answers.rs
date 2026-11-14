use std::time::{Duration, Instant};

use dam_application::{
    ConfiguredDuration, HelperError, MutationOp, MutationOutcome, MutationVerdict, PushOutcome,
    RemoteConfig, RemoteHelper, RemoteMutation,
};
use dam_domain::Oid;

use super::super::super::testing::{install, install_listening_to, remote};
use crate::Interrupt;

const ASKED: &str =
    "read -r line\ncase \"$line\" in *'\"early_answers\":true'*) ;; *) exit 3 ;; esac\n";

fn early(n: u8) -> String {
    format!(
        "printf '{{\"early_answer\":{{\"oid\":\"{}\",\"ok\":true,\"remote_id\":\"r{n}\"}}}}\\n'\n",
        format!("{n:02}").repeat(20)
    )
}

fn script(steps: &[&str]) -> String {
    format!("#!/bin/sh\n{ASKED}{}", steps.concat())
}

const HANG: &str = "while :; do sleep 0.05; done\n";

fn oid(n: u8) -> Oid {
    Oid::parse(&format!("{n:02}").repeat(20)).unwrap()
}

fn three_deletes() -> Vec<RemoteMutation> {
    (1..=3)
        .map(|n| RemoteMutation {
            op: MutationOp::Delete,
            oid: oid(n),
            idempotency_key: format!("k{n}"),
            remote_id: Some(format!("r{n}")),
            container_remote_id: None,
            object: None,
            fields: vec![],
        })
        .collect()
}

fn accepted(n: u8) -> MutationOutcome {
    MutationOutcome {
        oid: oid(n),
        verdict: MutationVerdict::Accepted {
            remote_id: Some(format!("r{n}")),
        },
    }
}

fn silent_for_at_most(millis: u64) -> RemoteConfig {
    let mut remote = remote();
    remote.deadline = Some(ConfiguredDuration {
        value: Duration::from_millis(millis),
        text: format!("{millis}ms"),
    });
    remote
}

#[test]
fn a_helper_that_hangs_after_two_answers_hands_them_back_with_the_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), &script(&[&early(1), &early(2), HANG]));
    let mut helper = launcher.spawn(&silent_for_at_most(1500), &[]).unwrap();
    let pushed = helper.push(three_deletes()).unwrap();
    assert_eq!(
        pushed,
        PushOutcome {
            outcomes: vec![accepted(1), accepted(2)],
            stopped_by: Some(HelperError::Timeout {
                helper: "t".into(),
                deadline: "1500ms".into(),
                said: None,
            }),
        }
    );
    assert!(
        helper.child.try_wait().unwrap().is_some(),
        "the helper was killed"
    );
}

#[test]
fn a_helper_that_exits_after_two_answers_hands_them_back_with_the_failure() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), &script(&[&early(1), &early(2), "exit 1\n"]));
    let mut helper = launcher.spawn(&silent_for_at_most(5000), &[]).unwrap();
    let pushed = helper.push(three_deletes()).unwrap();
    assert_eq!(pushed.outcomes, vec![accepted(1), accepted(2)]);
    assert!(
        matches!(pushed.stopped_by, Some(HelperError::Io(_))),
        "{:?}",
        pushed.stopped_by
    );
}

#[test]
fn an_interrupt_after_two_answers_hands_them_back_with_the_cancel() {
    let dir = tempfile::tempdir().unwrap();
    let interrupt = Interrupt::default();
    let marker = dir.path().join("answered-two");
    let touch = format!("touch '{}'\n", marker.display());
    let launcher = install_listening_to(
        dir.path(),
        &script(&[&early(1), &early(2), &touch, HANG]),
        &interrupt,
    );
    let mut helper = launcher.spawn(&silent_for_at_most(5000), &[]).unwrap();
    let pressing = interrupt.clone();
    std::thread::spawn(move || {
        let give_up_at = Instant::now() + Duration::from_secs(10);
        while !marker.exists() && Instant::now() < give_up_at {
            std::thread::sleep(Duration::from_millis(10));
        }
        pressing.request();
    });
    let pushed = helper.push(three_deletes()).unwrap();
    assert_eq!(
        pushed,
        PushOutcome {
            outcomes: vec![accepted(1), accepted(2)],
            stopped_by: Some(HelperError::Cancelled),
        }
    );
}

#[test]
fn a_helper_that_answers_once_is_read_as_before() {
    let dir = tempfile::tempdir().unwrap();
    let once = format!(
        "#!/bin/sh\nread -r line\nprintf '{{\"results\":[{{\"oid\":\"{}\",\"ok\":true,\"remote_id\":\"r1\"}}]}}\\n'\n{HANG}",
        "01".repeat(20)
    );
    let launcher = install(dir.path(), &once);
    let mut helper = launcher.spawn(&silent_for_at_most(5000), &[]).unwrap();
    assert_eq!(
        helper.push(three_deletes()).unwrap(),
        PushOutcome {
            outcomes: vec![accepted(1)],
            stopped_by: None,
        }
    );
}

#[test]
fn a_helper_answering_steadily_is_not_cut_off_when_the_whole_call_outlasts_the_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let pause = "sleep 0.5\n";
    let results: Vec<String> = (1..=3)
        .map(|n| {
            format!(
                "{{\"oid\":\"{}\",\"ok\":true,\"remote_id\":\"r{n}\"}}",
                format!("{n:02}").repeat(20)
            )
        })
        .collect();
    let reply = format!("printf '{{\"results\":[{}]}}\\n'\n", results.join(","));
    let steps = [
        pause,
        &early(1),
        pause,
        &early(2),
        pause,
        &early(3),
        pause,
        &reply,
    ];
    let launcher = install(dir.path(), &script(&steps));
    let mut helper = launcher.spawn(&silent_for_at_most(1500), &[]).unwrap();
    let started = Instant::now();
    let pushed = helper.push(three_deletes()).unwrap();
    assert!(
        started.elapsed() > Duration::from_millis(1500),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(
        pushed,
        PushOutcome {
            outcomes: vec![accepted(1), accepted(2), accepted(3)],
            stopped_by: None,
        }
    );
}
