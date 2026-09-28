use super::*;
use crate::args::DoneArgs;
use crate::testing::{ScriptedPrompt, context, machine_context};
use dam_application::{ChildDisposition, DependencyDisposition, Refusal};
use dam_domain::{Object, Oid, Path, Task};
use std::cell::RefCell;

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn parent_and_child(ctx: &Context) {
    let mut p = Task::new(oid(1), "parent");
    p.base.path = Path::parse("p").unwrap();
    let mut c = Task::new(oid(2), "child");
    c.base.path = Path::parse("p/c").unwrap();
    ctx.store.put(&Object::Task(p)).unwrap();
    ctx.store.put(&Object::Task(c)).unwrap();
}

const MOVE_THEM_UP_ONE_LEVEL: usize = 0;

fn path_of(ctx: &Context, b: u8) -> String {
    ctx.store
        .get(&oid(b))
        .unwrap()
        .unwrap()
        .base()
        .path
        .as_str()
        .to_string()
}

fn a_prompt_with_no_answers_so_any_question_cancels() -> ScriptedPrompt {
    ScriptedPrompt {
        choices: RefCell::new(vec![]),
        texts: RefCell::new(vec![]),
    }
}

fn args(oid: &Oid, force: bool, interactive: bool) -> DoneArgs {
    DoneArgs {
        oid: oid.as_str().into(),
        force,
        interactive,
        children: None,
        depends: None,
    }
}

fn parent_child_and_dependency(ctx: &Context) {
    parent_and_child(ctx);
    let blocker = Task::new(oid(3), "blocker");
    ctx.store.put(&Object::Task(blocker)).unwrap();
    let mut parent = ctx.store.get(&oid(1)).unwrap().unwrap();
    parent.base_mut().depends.push(oid(3));
    ctx.store.put(&parent).unwrap();
}

#[test]
fn a_blocked_task_is_refused_without_force() {
    let mut ctx = context();
    parent_and_child(&ctx);
    let err = run(&mut ctx, args(&oid(1), false, false)).unwrap_err();
    assert_eq!(err.exit_code(), 4, "a refusal by dam's own rule");
    assert!(err.to_string().contains("child"));
}

#[test]
fn force_marks_it_done_and_prints_done() {
    let mut ctx = context();
    parent_and_child(&ctx);
    let report = run(&mut ctx, args(&oid(1), true, false)).unwrap();
    assert_eq!(report.human, "done");
    assert!(
        ctx.store
            .get(&oid(1))
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .done
    );
}

#[test]
fn interactive_asks_and_moves_children_up() {
    let mut ctx = context();
    parent_and_child(&ctx);
    ctx.prompt = Box::new(ScriptedPrompt {
        choices: RefCell::new(vec![MOVE_THEM_UP_ONE_LEVEL]),
        texts: RefCell::new(vec![]),
    });
    run(&mut ctx, args(&oid(1), true, true)).unwrap();
    assert_eq!(
        path_of(&ctx, 2),
        "c/",
        "up keeps the child's own name one level up, beside its former parent"
    );
}

#[test]
fn config_interactive_applies_to_a_plain_force() {
    let mut ctx = context();
    ctx.config.done_interactive = true;
    parent_and_child(&ctx);
    ctx.prompt = Box::new(ScriptedPrompt {
        choices: RefCell::new(vec![MOVE_THEM_UP_ONE_LEVEL]),
        texts: RefCell::new(vec![]),
    });
    run(&mut ctx, args(&oid(1), true, false)).unwrap();
    assert_eq!(
        path_of(&ctx, 2),
        "c/",
        "the child moved, so the prompt was asked rather than skipped"
    );
}

#[test]
fn a_machine_format_refuses_to_ask_instead_of_blocking() {
    let mut ctx = machine_context();
    parent_and_child(&ctx);
    let err = run(&mut ctx, args(&oid(1), true, true)).unwrap_err();
    assert_eq!(err.to_string(), Refusal::NeedsAnAnswer.to_string());
    assert_eq!(err.exit_code(), 4);
    assert!(
        !ctx.store
            .get(&oid(1))
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .done
    );
}

#[test]
fn dispositions_complete_a_blocked_parent_without_a_terminal() {
    let mut ctx = machine_context();
    parent_and_child(&ctx);
    let report = run(
        &mut ctx,
        DoneArgs {
            children: Some(ChildDisposition::Keep),
            depends: Some(DependencyDisposition::Drop),
            ..args(&oid(1), true, false)
        },
    )
    .unwrap();
    assert_eq!(report.human, "done");
    assert!(
        ctx.store
            .get(&oid(1))
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .done
    );
    assert_eq!(
        ctx.store
            .get(&oid(2))
            .unwrap()
            .unwrap()
            .base()
            .path
            .as_str(),
        "p/c/",
        "keep leaves the child where it is"
    );
}

#[test]
fn a_disposition_flag_asks_nothing_even_when_config_says_interactive() {
    let mut ctx = context();
    ctx.config.done_interactive = true;
    parent_and_child(&ctx);
    ctx.prompt = Box::new(a_prompt_with_no_answers_so_any_question_cancels());
    run(
        &mut ctx,
        DoneArgs {
            children: Some(ChildDisposition::Up),
            ..args(&oid(1), true, false)
        },
    )
    .unwrap();
    assert_eq!(
        ctx.store
            .get(&oid(2))
            .unwrap()
            .unwrap()
            .base()
            .path
            .as_str(),
        "c/"
    );
}

#[test]
fn the_flag_that_is_absent_keeps_what_is_there() {
    let mut ctx = machine_context();
    parent_child_and_dependency(&ctx);
    run(
        &mut ctx,
        DoneArgs {
            children: Some(ChildDisposition::Up),
            depends: None,
            ..args(&oid(1), true, false)
        },
    )
    .unwrap();
    assert_eq!(
        ctx.store.get(&oid(1)).unwrap().unwrap().base().depends,
        vec![oid(3)],
        "the dependency default is keep, so the edge survives"
    );
}
