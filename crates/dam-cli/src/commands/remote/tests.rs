use super::*;
use crate::args::{RemoteArgs, RemoteCommand};
use crate::testing::{FailingLauncher, context};
use dam_adapters::load_config;
use dam_application::{RemoteConfig, RemoteName};
use std::time::Duration;

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
    run_remote(&ctx, add("fake", "fake::")).unwrap();
    let before = std::fs::read(&ctx.config_path).unwrap();
    for name in ["a.b", "bad name", "x]"] {
        let err = run_remote(&ctx, add(name, "t::")).unwrap_err();
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
    run_remote(&ctx, add("fake", "fake::")).unwrap();
    ctx.config = load_config(&ctx.config_path).unwrap();
    let err = run_remote(&ctx, add("fake", "fake::")).unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert_eq!(err.to_string(), "remote fake already exists");
    assert_eq!(err.document()["error"]["rule"], "remote_exists");
}

#[test]
fn remote_add_writes_the_config_and_the_next_run_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = context();
    ctx.config_path = dir.path().join("config.toml");
    run_remote(&ctx, add("todoist", "todoist::default")).unwrap();
    ctx.config = load_config(&ctx.config_path).unwrap();
    let report = run_remote(
        &ctx,
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
    refresh_stale_unless_no_pull(&ctx).unwrap();
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
