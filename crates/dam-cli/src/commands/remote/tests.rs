use super::*;
use crate::args::{RemoteArgs, RemoteCommand};
use crate::testing::{EchoLauncher, FailingLauncher, context};
use dam_adapters::load_config;
use dam_application::{IncomingObject, PullOutcome};
use dam_application::{RemoteConfig, RemoteName};
use dam_domain::{Object, Oid, Task};
use std::cell::RefCell;
use std::time::Duration;

fn incoming(tag: &str, subject: &str) -> IncomingObject {
    IncomingObject {
        remote_id: format!("r-{tag}"),
        object: Object::Task(Task::new(
            Oid::generate(&mut |b: &mut [u8]| b.fill(0)),
            subject,
        )),
    }
}

fn add(name: &str, url: &str) -> RemoteArgs {
    RemoteArgs {
        command: RemoteCommand::Add {
            name: name.into(),
            url: url.into(),
        },
    }
}

#[test]
fn remote_add_refuses_a_name_outside_the_bare_key_set_and_leaves_the_config_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = context();
    ctx.config_path = dir.path().join("config.toml");
    run_remote(&mut ctx, add("fake", "fake::")).unwrap();
    let before = std::fs::read(&ctx.config_path).unwrap();
    for name in ["a.b", "bad name", "x]"] {
        let err = run_remote(&mut ctx, add(name, "t::")).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{name}");
        assert_eq!(
            err.to_string(),
            format!("remote name {name:?} may hold only letters, digits, - and _")
        );
    }
    assert_eq!(std::fs::read(&ctx.config_path).unwrap(), before);
}

#[test]
fn remote_add_refuses_a_duplicate_as_a_rule() {
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = context();
    ctx.config_path = dir.path().join("config.toml");
    run_remote(&mut ctx, add("fake", "fake::")).unwrap();
    ctx.config = load_config(&ctx.config_path).unwrap();
    let err = run_remote(&mut ctx, add("fake", "fake::")).unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert_eq!(err.to_string(), "remote fake already exists");
    assert_eq!(err.document()["error"]["rule"], "remote_exists");
}

#[test]
fn remote_add_writes_the_config_and_the_next_run_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = context();
    ctx.config_path = dir.path().join("config.toml");
    run_remote(&mut ctx, add("todoist", "todoist::default")).unwrap();
    ctx.config = load_config(&ctx.config_path).unwrap();
    let report = run_remote(
        &mut ctx,
        RemoteArgs {
            command: RemoteCommand::List,
        },
    )
    .unwrap();
    assert!(
        report.human.contains("todoist::default"),
        "list must echo the url as configured, not just the helper prefix: {}",
        report.human
    );
    assert_eq!(ctx.config.remote("todoist").unwrap().helper, "todoist");
}

#[test]
fn a_stale_remote_is_pulled_before_a_read_and_a_fresh_one_is_not() {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        stale: Some(Duration::from_secs(60)),
        deadline: None,
        path: None,
    });
    ctx.launcher = Box::new(EchoLauncher {
        pulled: RefCell::new(Some(PullOutcome {
            objects: vec![incoming("aaaa", "from upstream")],
            ..PullOutcome::default()
        })),
    });
    maybe_pull_stale(&mut ctx).unwrap();
    assert_eq!(ctx.store.all().unwrap().len(), 1);
    ctx.launcher = Box::new(EchoLauncher {
        pulled: RefCell::new(Some(PullOutcome {
            objects: vec![incoming("bbbb", "second")],
            ..PullOutcome::default()
        })),
    });
    maybe_pull_stale(&mut ctx).unwrap();
    assert_eq!(
        ctx.store.all().unwrap().len(),
        1,
        "a fresh remote is not pulled again"
    );
}

#[test]
fn no_pull_answers_from_the_store_and_never_launches_a_helper() {
    let mut ctx = context();
    ctx.no_pull = true;
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        stale: Some(Duration::from_secs(60)),
        deadline: None,
        path: None,
    });
    ctx.launcher = Box::new(FailingLauncher);
    maybe_pull_stale(&mut ctx).unwrap();
    assert!(ctx.store.all().unwrap().is_empty());
    assert!(
        ctx.store.notices().unwrap().is_empty(),
        "reaching for the unreachable helper at all would have left a PullFailed notice"
    );
    assert!(
        ctx.store
            .last_pull(&RemoteName("t".into()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_remote_without_stale_is_never_auto_pulled() {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        stale: None,
        deadline: None,
        path: None,
    });
    ctx.launcher = Box::new(EchoLauncher {
        pulled: RefCell::new(Some(PullOutcome {
            objects: vec![incoming("aaaa", "x")],
            ..PullOutcome::default()
        })),
    });
    maybe_pull_stale(&mut ctx).unwrap();
    assert!(ctx.store.all().unwrap().is_empty());
}

#[test]
fn an_unreachable_stale_remote_records_a_notice_and_does_not_advance_last_pull() {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        stale: Some(Duration::from_secs(60)),
        deadline: None,
        path: None,
    });
    ctx.launcher = Box::new(FailingLauncher);
    maybe_pull_stale(&mut ctx).unwrap();
    assert!(ctx.store.all().unwrap().is_empty());
    assert!(
        ctx.store
            .last_pull(&RemoteName("t".into()))
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        ctx.store.notices().unwrap().last(),
        Some(dam_application::Notice::PullFailed { remote, .. }) if remote.0 == "t"
    ));
}
