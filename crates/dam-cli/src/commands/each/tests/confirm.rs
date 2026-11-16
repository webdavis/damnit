use std::cell::RefCell;

use dam_domain::Permission;

use super::*;
use crate::error::Problem;
use crate::testing::{ScriptedPrompt, holding, machine_context};

pub(super) fn asking_above(limit: usize, answers: &[&str]) -> (Context, Vec<String>) {
    let (mut ctx, oids) = three_tasks();
    ctx.config.confirm_above = Some(limit);
    ctx.prompt = Box::new(ScriptedPrompt {
        choices: RefCell::new(vec![]),
        texts: RefCell::new(answers.iter().rev().map(|a| a.to_string()).collect()),
    });
    (ctx, oids)
}

pub(super) fn subject(ctx: &Context, oid: &str) -> String {
    let oid = Oid::parse(oid).unwrap();
    ctx.store.get(&oid).unwrap().unwrap().base().subject.clone()
}

const ALL: &[&str] = &["edit", "--filter", "tasks", "--subject", "x"];

#[test]
fn yes_at_the_question_changes_the_set() {
    let (ctx, oids) = asking_above(2, &["y"]);
    let out = dam(&ctx, ALL).unwrap();
    assert_eq!(last_line(&out), "changed 3 of 3");
    assert_eq!(subject(&ctx, &oids[0]), "x");
}

#[test]
fn no_at_the_question_changes_nothing_and_exits_3() {
    let (ctx, oids) = asking_above(2, &["n"]);
    let refused = dam(&ctx, ALL).unwrap_err();
    assert_eq!(refused.exit_code(), 3);
    assert_eq!(subject(&ctx, &oids[0]), "t1");
}

#[test]
fn a_set_at_or_below_the_setting_is_not_asked() {
    let (ctx, _) = asking_above(3, &[]);
    assert_eq!(last_line(&dam(&ctx, ALL).unwrap()), "changed 3 of 3");
}

#[test]
fn without_the_setting_nothing_is_asked() {
    let (ctx, _) = three_tasks();
    assert_eq!(last_line(&dam(&ctx, ALL).unwrap()), "changed 3 of 3");
}

#[test]
fn named_oids_a_preview_and_an_expect_are_not_asked() {
    let (ctx, oids) = asking_above(1, &[]);
    dam(
        &ctx,
        &["edit", &oids[0], &oids[1], &oids[2], "--subject", "x"],
    )
    .unwrap();
    dam(&ctx, &["done", "--filter", "tasks", "--preview"]).unwrap();
    dam(
        &ctx,
        &[
            "edit",
            "--filter",
            "tasks",
            "--subject",
            "y",
            "--expect",
            "3",
        ],
    )
    .unwrap();
    assert_eq!(subject(&ctx, &oids[2]), "y");
}

#[test]
fn off_a_terminal_it_needs_expect() {
    let (mut ctx, oids) = three_tasks();
    ctx.prompt = machine_context().prompt;
    ctx.config.confirm_above = Some(2);
    let refused = dam(&ctx, ALL).unwrap_err();
    assert_eq!(refused.exit_code(), 2);
    let CliError::CommandLine(misuse) = &refused else {
        panic!("{refused:?}")
    };
    assert_eq!(misuse.problem, Problem::MissingArgument);
    assert_eq!(misuse.word.as_deref(), Some("--expect"));
    assert_eq!(subject(&ctx, &oids[0]), "t1");
}

#[test]
fn a_token_is_never_asked() {
    let (mut ctx, _) = asking_above(1, &[]);
    ctx.access = holding(&[Permission::TaskRead, Permission::TaskChange]);
    let out = dam(&ctx, ALL).unwrap();
    assert_eq!(out.data["count"], 3);
}
