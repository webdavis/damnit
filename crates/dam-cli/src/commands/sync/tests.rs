use super::*;
use crate::args::{AddArgs, Cli, Command, CommitArgs, PullArgs, PushArgs};
use crate::commands::commit::run_commit;
use crate::commands::stage::run_add;
use crate::testing::{EchoLauncher, context};
use clap::Parser;
use dam_application::{IncomingObject, PullOutcome};
use dam_application::{RemoteConfig, RemoteName};
use dam_domain::{Object, Oid, Task};
use std::cell::RefCell;

fn with_remote() -> Context {
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
    ctx
}

#[test]
fn push_sends_unpushed_commits_and_reports_counts() {
    let ctx = with_remote();
    ctx.store
        .put(&Object::Task(Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
            "a",
        )))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &ctx,
        CommitArgs {
            message: "m".into(),
        },
    )
    .unwrap();
    let report = run_push(&ctx, PushArgs { remote: None }).unwrap();
    assert!(report.human.contains("t: 1 sent, 1 ok"), "{}", report.human);
    assert_eq!(report.data["remotes"][0]["succeeded"], 1);
    assert!(
        ctx.store
            .unpushed(&RemoteName("t".into()))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn pull_reports_what_arrived_and_resolve_settles_a_conflict() {
    let mut ctx = with_remote();
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
    ctx.store
        .put(&Object::Task(Task::new(oid.clone(), "mine")))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &ctx,
        CommitArgs {
            message: "m".into(),
        },
    )
    .unwrap();
    run_push(&ctx, PushArgs { remote: None }).unwrap();
    let mut mine = Task::new(oid.clone(), "mine, edited");
    mine.base.subject = "mine, edited".into();
    ctx.store.put(&Object::Task(mine)).unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &ctx,
        CommitArgs {
            message: "edit".into(),
        },
    )
    .unwrap();
    let theirs = IncomingObject {
        remote_id: format!("r-{}", &oid.as_str()[..4]),
        object: Object::Task(Task::new(oid.clone(), "theirs")),
    };
    ctx.launcher = Box::new(EchoLauncher {
        pulled: RefCell::new(Some(PullOutcome {
            objects: vec![theirs],
            ..PullOutcome::default()
        })),
    });
    let report = run_pull(
        &ctx,
        PullArgs {
            remote: Some("t".into()),
        },
    )
    .unwrap();
    assert_eq!(report.data["remotes"][0]["conflicts"], 1);
    assert_eq!(ctx.store.conflicts().unwrap().len(), 1);
    let Command::Resolve(args) = Cli::try_parse_from(["dam", "resolve", oid.short(), "--theirs"])
        .unwrap()
        .command
    else {
        panic!("wrong command")
    };
    run_resolve(&ctx, args).unwrap();
    assert!(ctx.store.conflicts().unwrap().is_empty());
    assert_eq!(
        ctx.store.get(&oid).unwrap().unwrap().base().subject,
        "theirs"
    );
}
