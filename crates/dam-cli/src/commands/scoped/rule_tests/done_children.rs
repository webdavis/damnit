use dam_application::testing::task_in;
use dam_application::{ChildDisposition, ProjectSettings};
use dam_domain::{Object, Oid};

use super::super::dispatch_in;
use super::inside;
use crate::args::{Command, DoneArgs};
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::machine_context;

fn oid(byte: u8) -> Oid {
    task_in(byte, "", "").base.oid
}

fn ruled(rule: ChildDisposition) -> Standing {
    inside(ProjectSettings {
        done_children: Some(rule),
        ..ProjectSettings::default()
    })
}

fn holding() -> Context {
    let ctx = machine_context();
    for task in [
        task_in(1, "parent", "app/top"),
        task_in(2, "child", "app/top/parent"),
        task_in(3, "other", "home"),
        task_in(4, "kid", "home/other"),
    ] {
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

fn done(oids: &[u8], force: bool) -> DoneArgs {
    DoneArgs {
        oid: oids.iter().map(|b| oid(*b).as_str().to_string()).collect(),
        set: Default::default(),
        force,
        interactive: false,
        children: None,
        depends: None,
        scope: Default::default(),
        project_children: None,
    }
}

fn path_of(ctx: &Context, byte: u8) -> String {
    let object = ctx.store.get(&oid(byte)).unwrap().unwrap();
    object.base().path.as_str().to_string()
}

fn after(rule: ChildDisposition, args: DoneArgs) -> Context {
    let ctx = holding();
    dispatch_in(&ctx, Command::Done(args), ruled(rule)).unwrap();
    ctx
}

#[test]
fn a_forced_done_under_the_project_path_follows_each_value_of_the_rule() {
    let ctx = after(ChildDisposition::Up, done(&[1], true));
    assert_eq!(path_of(&ctx, 2), "app/top/");
    let ctx = after(ChildDisposition::Keep, done(&[1], true));
    assert_eq!(path_of(&ctx, 2), "app/top/parent/");
    let ctx = after(ChildDisposition::Into("rest".into()), done(&[1], true));
    assert_eq!(path_of(&ctx, 2), "app/top/rest/");
}

#[test]
fn a_task_outside_the_project_path_keeps_its_children() {
    let ctx = after(ChildDisposition::Up, done(&[3], true));
    assert_eq!(path_of(&ctx, 4), "home/other/");
}

#[test]
fn children_on_the_command_line_wins_over_the_rule() {
    let args = DoneArgs {
        children: Some(ChildDisposition::Keep),
        ..done(&[1], true)
    };
    let ctx = after(ChildDisposition::Up, args);
    assert_eq!(path_of(&ctx, 2), "app/top/parent/");
}

#[test]
fn depends_alone_on_the_command_line_still_follows_the_rule_for_children() {
    let args = DoneArgs {
        depends: Some(dam_domain::DependencyDisposition::Drop),
        ..done(&[1], true)
    };
    let ctx = after(ChildDisposition::Up, args);
    assert_eq!(path_of(&ctx, 2), "app/top/");
}

#[test]
fn without_force_the_refusal_stands() {
    let ctx = holding();
    let refused = dispatch_in(
        &ctx,
        Command::Done(done(&[1], false)),
        ruled(ChildDisposition::Up),
    );
    assert_eq!(refused.unwrap_err().exit_code(), 4);
    assert_eq!(path_of(&ctx, 2), "app/top/parent/");
}

#[test]
fn global_leaves_the_rule_out() {
    let mut args = done(&[1], true);
    args.scope.global = true;
    let ctx = after(ChildDisposition::Up, args);
    assert_eq!(path_of(&ctx, 2), "app/top/parent/");
}

#[test]
fn outside_a_project_nothing_changes() {
    let ctx = holding();
    dispatch_in(&ctx, Command::Done(done(&[1], true)), Standing::Outside).unwrap();
    assert_eq!(path_of(&ctx, 2), "app/top/parent/");
}

#[test]
fn a_set_applies_the_rule_only_to_its_tasks_under_the_project_path() {
    let ctx = after(ChildDisposition::Up, done(&[1, 3], true));
    assert_eq!(path_of(&ctx, 2), "app/top/");
    assert_eq!(path_of(&ctx, 4), "home/other/");
}

#[test]
fn a_preview_of_a_set_holding_the_child_shows_where_the_rule_moves_it() {
    let mut args = done(&[1, 2], true);
    args.set.preview = true;
    let ctx = holding();
    let shown = dispatch_in(&ctx, Command::Done(args), ruled(ChildDisposition::Up)).unwrap();
    assert!(
        shown.human.contains("+child  done: true  app/top/\n"),
        "{}",
        shown.human
    );
    assert_eq!(path_of(&ctx, 2), "app/top/parent/");
}
