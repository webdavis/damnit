
use crate::args::{AddArgs, CommitArgs, DiffArgs};
use crate::commands::commit::run_commit;
use crate::commands::stage::run_add;
use crate::testing::context;
use dam_application::{CredentialSpec, RemoteConfig, RemoteName};
use dam_domain::{Kind, Object, Oid, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

#[test]
fn status_separates_staged_unstaged_and_notices() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "staged")))
        .unwrap();
    run_add(
        &mut ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    ctx.store
        .put(&Object::Task(Task::new(oid(2), "loose")))
        .unwrap();
    ctx.store
        .add_notice(&dam_application::Notice::PushFailed {
            remote: dam_application::RemoteName("t".into()),
            oid: oid(1),
            why: "nope".into(),
        })
        .unwrap();
    let report = super::run_status(&mut ctx).unwrap();
    assert!(report.human.contains("Staged:"));
    assert!(report.human.contains("Not staged:"));
    assert!(report.human.contains("Notices:"));
    assert!(report.human.contains("nope"));
    assert_eq!(report.data["staged"].as_array().unwrap().len(), 1);
    assert_eq!(report.data["unstaged"].as_array().unwrap().len(), 1);
    assert_eq!(report.data["notices"].as_array().unwrap().len(), 1);
}

fn remote_with(credentials: Vec<CredentialSpec>) -> RemoteConfig {
    RemoteConfig {
        name: RemoteName("todoist".into()),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials,
        stale: None,
        deadline: None,
        path: None,
    }
}

#[test]
fn status_warns_once_when_a_credential_is_a_value_in_the_config_file() {
    let mut ctx = context();
    ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Literal {
        name: "api_token".into(),
        value: "SUPERSECRETTOKEN".into(),
    }])];
    let report = super::run_status(&mut ctx).unwrap();
    let warnings: Vec<&str> = report
        .human
        .lines()
        .filter(|l| l.starts_with("warning:"))
        .collect();
    assert_eq!(warnings.len(), 1, "{}", report.human);
    assert!(warnings[0].contains("todoist.api_token"), "{}", warnings[0]);
    assert!(!report.human.contains("SUPERSECRETTOKEN"));
}

#[test]
fn status_does_not_warn_when_no_credential_is_a_value_in_the_config_file() {
    let mut ctx = context();
    ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Env {
        name: "api_token".into(),
        var: "DAM_TODOIST_API_TOKEN".into(),
    }])];
    let report = super::run_status(&mut ctx).unwrap();
    assert!(!report.human.contains("warning:"), "{}", report.human);
}

#[test]
fn a_kind_change_notice_names_both_kinds() {
    let mut ctx = context();
    ctx.store
        .add_notice(&dam_application::Notice::KindChanged {
            oid: oid(1),
            ours: Kind::Task,
            theirs: Kind::Event,
        })
        .unwrap();
    let report = super::run_status(&mut ctx).unwrap();
    assert!(
        report.human.contains(&format!(
            "{} is a task here and an event upstream",
            oid(1).short()
        )),
        "{}",
        report.human
    );
    assert_eq!(report.data["notices"][0]["kind"], "kind_changed");
    assert_eq!(report.data["notices"][0]["ours"], "task");
    assert_eq!(report.data["notices"][0]["theirs"], "event");
}

#[test]
fn a_kind_change_notice_reads_the_other_way_round_too() {
    let mut ctx = context();
    ctx.store
        .add_notice(&dam_application::Notice::KindChanged {
            oid: oid(1),
            ours: Kind::Event,
            theirs: Kind::Task,
        })
        .unwrap();
    let report = super::run_status(&mut ctx).unwrap();
    assert!(
        report.human.contains(&format!(
            "{} is an event here and a task upstream",
            oid(1).short()
        )),
        "{}",
        report.human
    );
}

#[test]
fn diff_shows_working_or_staged() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "a")))
        .unwrap();
    run_add(
        &mut ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &mut ctx,
        CommitArgs {
            message: "m".into(),
        },
    )
    .unwrap();
    let mut t = Task::new(oid(1), "b");
    t.base.subject = "b".into();
    ctx.store.put(&Object::Task(t)).unwrap();
    let working = super::run_diff(&mut ctx, DiffArgs { staged: false }).unwrap();
    assert!(working.human.contains("changed"));
    assert!(working.human.contains("subject"));
    let staged = super::run_diff(&mut ctx, DiffArgs { staged: true }).unwrap();
    assert!(staged.human.is_empty());
}

#[test]
fn status_reports_conflicts_and_unpushed_commits() {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("todoist".into()),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials: vec![],
        stale: None,
        deadline: None,
        path: None,
    });
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "mine")))
        .unwrap();
    ctx.store
        .mark_conflict(
            &RemoteName("todoist".into()),
            &oid(1),
            &Object::Task(Task::new(oid(1), "theirs")),
        )
        .unwrap();
    ctx.store
        .put(&Object::Task(Task::new(oid(2), "b")))
        .unwrap();
    run_add(
        &mut ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &mut ctx,
        CommitArgs {
            message: "m".into(),
        },
    )
    .unwrap();
    let report = super::run_status(&mut ctx).unwrap();
    assert!(report.human.contains("Conflicts:"));
    assert!(report.human.contains(oid(1).short()));
    assert!(report.human.contains("\"mine\""));
    assert!(report.human.contains("\"theirs\""));
    assert!(report.human.contains("Unpushed:"));
    assert!(report.human.contains("todoist: 1 commit(s)"));
    assert_eq!(report.data["conflicts"].as_array().unwrap().len(), 1);
    assert_eq!(report.data["conflicts"][0]["oid"], oid(1).to_string());
    assert_eq!(report.data["unpushed"][0]["remote"], "todoist");
    assert_eq!(report.data["unpushed"][0]["commits"], 1);
}
