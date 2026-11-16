use dam_application::{ConfigError, Project, ProjectSettings};
use dam_domain::{Object, Oid, Path, Task};

use super::dispatch_in;
use crate::args::{Command, MvArgs, SetArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::project_scope::Standing;
use crate::testing::context;

fn inside() -> Standing {
    Standing::Inside(Box::new(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings {
            path: "damnit/".into(),
            ..Default::default()
        },
    }))
}

fn unreadable() -> Standing {
    Standing::Unreadable(ConfigError::NotATable {
        path: "/work/damnit/.dam.lua".into(),
        returned: "a number".into(),
    })
}

fn filed(ctx: &Context, byte: u8, path: &str) -> String {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(byte));
    let mut task = Task::new(oid.clone(), format!("t{byte}"));
    task.base.path = Path::parse(path).unwrap();
    ctx.store.put(&Object::Task(task)).unwrap();
    oid.as_str().into()
}

fn mv(words: &[&str]) -> MvArgs {
    MvArgs {
        oid: words.iter().map(|w| w.to_string()).collect(),
        set: SetArgs::default(),
        path: None,
        parent: None,
        scope: Default::default(),
    }
}

fn paths(ctx: &Context) -> Vec<String> {
    let mut all: Vec<String> = ctx
        .store
        .all()
        .unwrap()
        .iter()
        .map(|o| o.base().path.to_string())
        .collect();
    all.sort();
    all
}

#[test]
fn one_oid_moves_under_the_project_inside_it() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    dispatch_in(&ctx, Command::Mv(mv(&[&oid, "cli/"])), inside()).unwrap();
    assert_eq!(paths(&ctx), ["damnit/cli/"]);
}

#[test]
fn the_path_flag_starts_from_the_project_too() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    let mut args = mv(&[&oid]);
    args.path = Some(Path::parse("cli/").unwrap());
    dispatch_in(&ctx, Command::Mv(args), inside()).unwrap();
    assert_eq!(paths(&ctx), ["damnit/cli/"]);
}

#[test]
fn several_oids_move_under_the_project() {
    let ctx = context();
    let (a, b) = (filed(&ctx, 1, "home/"), filed(&ctx, 2, "work/"));
    dispatch_in(&ctx, Command::Mv(mv(&[&a, &b, "cli/"])), inside()).unwrap();
    assert_eq!(paths(&ctx), ["damnit/cli/", "damnit/cli/"]);
}

#[test]
fn a_filter_moves_what_it_lists_under_the_project() {
    let mut ctx = context();
    ctx.config.filters = vec![dam_application::testing::saved(
        "tasks",
        &[("kind", "task")],
    )];
    filed(&ctx, 1, "damnit/");
    filed(&ctx, 2, "work/");
    let mut args = mv(&[]);
    args.path = Some(Path::parse("cli/").unwrap());
    args.set.filter = Some("tasks".into());
    dispatch_in(&ctx, Command::Mv(args), inside()).unwrap();
    assert_eq!(paths(&ctx), ["damnit/cli/", "work/"]);
}

#[test]
fn global_reads_the_path_from_the_root() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    let mut args = mv(&[&oid, "cli/"]);
    args.scope.global = true;
    dispatch_in(&ctx, Command::Mv(args), inside()).unwrap();
    assert_eq!(paths(&ctx), ["cli/"]);
}

#[test]
fn project_starts_the_path_from_that_project() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    let mut args = mv(&[&oid, "cli/"]);
    args.scope.project = Some(Path::parse("pns/").unwrap());
    dispatch_in(&ctx, Command::Mv(args), inside()).unwrap();
    assert_eq!(paths(&ctx), ["pns/cli/"]);
}

#[test]
fn outside_a_project_the_path_is_read_from_the_root() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    dispatch_in(&ctx, Command::Mv(mv(&[&oid, "cli/"])), Standing::Outside).unwrap();
    assert_eq!(paths(&ctx), ["cli/"]);
}

#[test]
fn an_unreadable_project_file_refuses_a_path_unless_global_is_given() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    let refused = dispatch_in(&ctx, Command::Mv(mv(&[&oid, "cli/"])), unreadable());
    assert!(matches!(refused, Err(CliError::Config(_))));
    assert_eq!(paths(&ctx), ["home/"]);
    let mut args = mv(&[&oid, "cli/"]);
    args.scope.global = true;
    dispatch_in(&ctx, Command::Mv(args), unreadable()).unwrap();
    assert_eq!(paths(&ctx), ["cli/"]);
}

#[test]
fn a_parent_alone_moves_as_it_does_outside_even_past_an_unreadable_project_file() {
    let ctx = context();
    let oid = filed(&ctx, 1, "home/");
    let parent = filed(&ctx, 2, "work/");
    let mut args = mv(&[&oid]);
    args.parent = Some(parent);
    dispatch_in(&ctx, Command::Mv(args), unreadable()).unwrap();
    assert_eq!(paths(&ctx), ["work/", "work/t2/"]);
}

#[test]
fn a_token_is_refused_before_the_path_is_read() {
    use dam_domain::Permission::{TaskMove, TaskRead};
    let ctx = Context {
        access: crate::testing::holding(&[TaskMove, TaskRead]),
        ..context()
    };
    let oid = filed(&ctx, 1, "home/");
    let refused = dispatch_in(&ctx, Command::Mv(mv(&[&oid, "cli/"])), unreadable());
    assert!(matches!(refused, Err(CliError::UseCase(_))));
    assert_eq!(paths(&ctx), ["home/"]);
}

#[test]
fn a_request_carries_global_and_project() {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        file.path(),
        r#"{"oid": "0101", "path": "cli/", "project": "pns/"}"#,
    )
    .unwrap();
    let line = vec![
        "dam".into(),
        "mv".into(),
        "--request".into(),
        file.path().into(),
    ];
    let Ok(cli) = crate::request::parse(line) else {
        panic!("refused")
    };
    let Command::Mv(args) = cli.command else {
        panic!("not mv")
    };
    assert_eq!(args.scope.project, Some(Path::parse("pns/").unwrap()));
}
