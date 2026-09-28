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
    let force = how_far_this_run_forces(ctx, &args, &plan.blockers)?;
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

fn how_far_this_run_forces(
    ctx: &Context,
    args: &DoneArgs,
    blockers: &[Blocker],
) -> Result<Force, CliError> {
    if !args.force {
        return Ok(Force::No);
    }
    if blockers.is_empty() {
        return Ok(Force::Yes);
    }
    match dispositions_answered_on_the_command_line_over_any_config(args) {
        Some(d) => Ok(Force::With(d)),
        None if args.interactive || ctx.config.done_interactive => {
            Ok(Force::With(ask(ctx, blockers)?))
        }
        None => Ok(Force::Yes),
    }
}

fn dispositions_answered_on_the_command_line_over_any_config(
    args: &DoneArgs,
) -> Option<Dispositions> {
    match (&args.children, args.depends) {
        (None, None) => None,
        (children, dependencies) => Some(Dispositions {
            children: children.clone().unwrap_or_default(),
            dependencies: dependencies.unwrap_or_default(),
        }),
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
mod tests;
