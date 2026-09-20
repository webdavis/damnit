use dam_application::{ConfiguredDuration, HelperError, RemoteHelper};

use super::super::testing::{install, remote};

#[test]
fn a_helper_that_dies_without_answering_quotes_what_it_complained_about() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\necho 'DAM_T_API_TOKEN is not set' >&2\nexit 1\n",
    );
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    let err = helper.capabilities().unwrap_err();
    let text = format!("{err:?}");
    assert!(text.contains("DAM_T_API_TOKEN is not set"), "{text}");
}

#[test]
fn a_helper_that_never_answers_carries_its_complaint_into_the_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\necho 'waiting on the upstream api' >&2\nwhile :; do sleep 0.05; done\n",
    );
    let mut remote = remote();
    remote.deadline = Some(ConfiguredDuration {
        value: std::time::Duration::from_millis(100),
        text: "100ms".into(),
    });
    let mut helper = launcher.spawn(&remote, &[]).unwrap();
    match helper.capabilities().unwrap_err() {
        HelperError::Timeout { said, .. } => {
            assert_eq!(said.as_deref(), Some("waiting on the upstream api"))
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn only_the_last_lines_of_a_talkative_helper_are_kept() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\ni=0\nwhile [ $i -lt 200 ]; do echo \"line $i\" >&2; i=$((i+1)); done\nexit 1\n",
    );
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    let text = format!("{:?}", helper.capabilities().unwrap_err());
    assert!(text.contains("line 199"), "{text}");
    assert!(!text.contains("line 0\\n"), "{text}");
    assert!(text.len() < 8000, "the tail is bounded, got {}", text.len());
}

#[test]
fn a_response_of_the_wrong_shape_names_the_shape_and_never_the_task_it_carried() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(
        dir.path(),
        "#!/bin/sh\nread -r line; printf '{\"objects\":[{\"oid\":\"01\",\"kind\":\"task\",\"subject\":\"call the clinic\",\"body\":\"ask about the referral\"}],\"removed\":[]}\\n'\n",
    );
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    let err = helper.capabilities().unwrap_err();
    let text = format!("{err:?}");
    assert!(text.contains("pull"), "{text}");
    assert!(!text.contains("call the clinic"), "{text}");
    assert!(!text.contains("referral"), "{text}");
}

#[test]
fn a_helper_declaring_a_newer_protocol_is_refused_with_both_versions() {
    let dir = tempfile::tempdir().unwrap();
    let newer = dam_protocol::PROTOCOL_VERSION + 1;
    let launcher = install(
        dir.path(),
        &format!(
            "#!/bin/sh\nread -r line; printf '{{\"protocol\":{newer},\"kinds\":[],\"fields\":[],\"credentials\":[],\"incremental\":false}}\\n'\n"
        ),
    );
    let mut helper = launcher.spawn(&remote(), &[]).unwrap();
    assert_eq!(
        helper.capabilities().unwrap_err(),
        HelperError::UnsupportedProtocol {
            helper: "t".into(),
            found: newer,
            supported: dam_protocol::PROTOCOL_VERSION,
        }
    );
}

#[test]
fn a_helper_that_never_answers_times_out_and_its_child_is_killed() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), "#!/bin/sh\nsleep 2\n");
    let mut remote = remote();
    remote.deadline = Some(ConfiguredDuration {
        value: std::time::Duration::from_millis(100),
        text: "100ms".into(),
    });
    let started = std::time::Instant::now();
    let mut helper = launcher.spawn(&remote, &[]).unwrap();
    let err = helper.capabilities().unwrap_err();
    assert_eq!(
        err,
        HelperError::Timeout {
            helper: "t".into(),
            deadline: "100ms".into(),
            said: None
        }
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    assert!(helper.child.try_wait().unwrap().is_some());
}

#[test]
fn the_timeout_echoes_the_configured_text_rather_than_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), "#!/bin/sh\nwhile :; do sleep 0.05; done\n");
    let mut remote = remote();
    remote.deadline = Some(ConfiguredDuration {
        value: std::time::Duration::from_millis(50),
        text: "2m".into(),
    });
    let mut helper = launcher.spawn(&remote, &[]).unwrap();
    assert_eq!(
        helper.capabilities().unwrap_err(),
        HelperError::Timeout {
            helper: "t".into(),
            deadline: "2m".into(),
            said: None
        }
    );
}

#[test]
fn a_helper_that_ignores_end_of_input_is_killed_at_drop() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = install(dir.path(), "#!/bin/sh\nwhile :; do sleep 0.05; done\n");
    let helper = launcher.spawn(&remote(), &[]).unwrap();
    let pid = helper.child.id().to_string();
    // Dropped on a worker so a regression to an unbounded wait reddens the
    // suite within the second instead of hanging it.
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&dropped);
    std::thread::spawn(move || {
        drop(helper);
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    let give_up_at = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while !dropped.load(std::sync::atomic::Ordering::SeqCst) {
        if std::time::Instant::now() >= give_up_at {
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid])
                .status();
            panic!("the launcher was still dropping a second later");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        !std::process::Command::new("kill")
            .args(["-0", &pid])
            .output()
            .unwrap()
            .status
            .success(),
        "the helper outlived its launcher"
    );
}
