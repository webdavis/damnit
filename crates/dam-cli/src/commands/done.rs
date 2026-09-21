use dam_application::{
    Blocker, ChildDisposition, Completed, DependencyDisposition, Dispositions, Force, complete,
    plan_complete,
};

use crate::args::DoneArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::Report;

pub(crate) fn run(ctx: &mut Context, args: DoneArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let plan = plan_complete(ctx.store.as_ref(), &oid)?;
    let force = force_for(ctx, &args, &plan.blockers)?;
    let outcome = complete(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &oid,
        force,
    )?;
    let human = match outcome {
        Completed::Done => "done".to_string(),
        Completed::RolledForward { next_due } => format!("rolled forward to {next_due}"),
    };
    Ok(Report {
        data: serde_json::json!({ "oid": oid.to_string(), "result": human }),
        human,
    })
}

/// How far this invocation forces. A disposition flag answers the prompt
/// without a terminal, so it is taken whatever `done.interactive` says; the
/// flag that is absent takes `Keep`, the answer that disturbs least, which is
/// the flags' own default: `ask` demands an answer for every blocker it finds.
fn force_for(ctx: &Context, args: &DoneArgs, blockers: &[Blocker]) -> Result<Force, CliError> {
    if !args.force {
        return Ok(Force::No);
    }
    let given = match (&args.children, args.depends) {
        (None, None) => None,
        (children, dependencies) => Some(Dispositions {
            children: children.clone().unwrap_or_default(),
            dependencies: dependencies.unwrap_or_default(),
        }),
    };
    if blockers.is_empty() {
        return Ok(Force::Yes);
    }
    match given {
        Some(d) => Ok(Force::With(d)),
        None if args.interactive || ctx.config.done_interactive => {
            Ok(Force::With(ask(ctx, blockers)?))
        }
        None => Ok(Force::Yes),
    }
}

fn ask(ctx: &Context, blockers: &[Blocker]) -> Result<Dispositions, CliError> {
    let children = blockers
        .iter()
        .filter(|b| matches!(b, Blocker::OpenChild(_)))
        .count();
    let deps = blockers
        .iter()
        .filter(|b| matches!(b, Blocker::OpenDependency(_)))
        .count();
    let children = if children == 0 {
        ChildDisposition::Keep
    } else {
        match ctx.prompt.choose(
            &format!("{children} open child task(s). What happens to them?"),
            &[
                "move them up one level",
                "move them into a new group",
                "keep them where they are",
            ],
        )? {
            0 => ChildDisposition::Up,
            1 => ChildDisposition::Into(ctx.prompt.text("name for the group:")?),
            _ => ChildDisposition::Keep,
        }
    };
    let dependencies = if deps == 0 {
        DependencyDisposition::Keep
    } else {
        match ctx.prompt.choose(
            &format!("{deps} open dependenc(ies). What happens to the links?"),
            &["drop them", "keep them"],
        )? {
            0 => DependencyDisposition::Drop,
            _ => DependencyDisposition::Keep,
        }
    };
    Ok(Dispositions {
        children,
        dependencies,
    })
}

#[cfg(test)]
mod tests {
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

    fn args(oid: &Oid, force: bool, interactive: bool) -> DoneArgs {
        DoneArgs {
            oid: oid.as_str().into(),
            force,
            interactive,
            children: None,
            depends: None,
        }
    }

    /// A parent with one open child and one open dependency, so both halves of
    /// the prompt have something to decide.
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
            choices: RefCell::new(vec![0]),
            texts: RefCell::new(vec![]),
        });
        run(&mut ctx, args(&oid(1), true, true)).unwrap();
        // Up keeps the child's own name one level up, beside its former
        // parent; see dam-application's complete::tests::interactive_up_*.
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
    fn config_interactive_applies_to_a_plain_force() {
        let mut ctx = context();
        ctx.config.done_interactive = true;
        parent_and_child(&ctx);
        // "move them up one level", an answer that moves the child, so the
        // assertion tells a prompt that was asked from one that was skipped.
        ctx.prompt = Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![0]),
            texts: RefCell::new(vec![]),
        });
        run(&mut ctx, args(&oid(1), true, false)).unwrap();
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
        // No scripted answer, so any question would come back cancelled.
        ctx.prompt = Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![]),
            texts: RefCell::new(vec![]),
        });
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
}
