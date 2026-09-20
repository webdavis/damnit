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
    let interactive = args.force && (args.interactive || ctx.config.done_interactive);
    let (force, dispositions) = match (args.force, interactive, plan.blockers.is_empty()) {
        (false, _, _) => (Force::No, None),
        (true, false, _) | (true, true, true) => (Force::Yes, None),
        (true, true, false) => (Force::Interactive, Some(ask(ctx, &plan.blockers)?)),
    };
    let outcome = complete(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_mut(),
        &oid,
        force,
        dispositions,
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
        }
    }

    #[test]
    fn a_blocked_task_is_refused_without_force() {
        let mut ctx = context();
        parent_and_child(&ctx);
        let err = run(&mut ctx, args(&oid(1), false, false)).unwrap_err();
        assert_eq!(err.exit_code(), 2);
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
        ctx.prompt = Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![2]),
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
            "p/c/"
        );
    }

    #[test]
    fn a_machine_format_refuses_to_ask_instead_of_blocking() {
        let mut ctx = machine_context();
        parent_and_child(&ctx);
        let err = run(&mut ctx, args(&oid(1), true, true)).unwrap_err();
        assert!(matches!(err, CliError::Usage(_)));
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
}
