use super::*;
use crate::args::{RemoteArgs, RemoteCommand};
use crate::testing::{FailingLauncher, FakeConfigFile, context};
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

fn remote(name: &str) -> RemoteConfig {
    RemoteConfig {
        name: RemoteName(name.into()),
        helper: name.into(),
        url: format!("{name}::"),
        credentials: vec![],
        stale: Some(Duration::from_secs(60)),
        deadline: None,
        path: None,
    }
}

fn context_writing_to(file: &FakeConfigFile) -> Context {
    Context {
        config_file: Box::new(file.clone()),
        ..context()
    }
}

#[test]
fn remote_add_refuses_a_name_outside_the_bare_key_set_and_leaves_the_config_as_it_was() {
    let file = FakeConfigFile::default();
    let ctx = context_writing_to(&file);
    for name in ["a.b", "bad name", "x]"] {
        let err = run_remote(&ctx, add(name, "t::")).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{name}");
        assert_eq!(
            err.to_string(),
            format!("remote name {name:?} may hold only letters, digits, - and _")
        );
    }
    assert_eq!(file.added(), vec![]);
}

#[test]
fn remote_add_refuses_a_duplicate_as_a_rule() {
    let file = FakeConfigFile::default();
    let mut ctx = context_writing_to(&file);
    ctx.config.remotes.push(remote("fake"));
    let err = run_remote(&ctx, add("fake", "fake::")).unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert_eq!(err.to_string(), "remote fake already exists");
    assert_eq!(err.document()["error"]["rule"], "remote_exists");
    assert_eq!(file.added(), vec![]);
}

#[test]
fn remote_add_writes_the_remote_to_the_config_file() {
    let file = FakeConfigFile::default();
    let ctx = context_writing_to(&file);
    let report = run_remote(&ctx, add("todoist", "todoist::default")).unwrap();
    assert_eq!(report.human, "added todoist (todoist::default)");
    assert_eq!(
        file.added(),
        vec![("todoist".to_string(), "todoist::default".to_string())]
    );
}

#[test]
fn no_pull_answers_from_the_store_and_never_launches_a_helper() {
    let mut ctx = context();
    ctx.no_pull = true;
    ctx.config.remotes.push(remote("t"));
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
