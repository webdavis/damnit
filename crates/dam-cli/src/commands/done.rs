use dam_application::{
    Blockers, ChildDisposition, Completed, DependencyDisposition, Dispositions, Force, Tracked,
    complete, plan_complete, resolve_object,
};

use crate::args::DoneArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, report};
use dam_domain::Oid;
use serde::Serialize;

#[derive(Serialize)]
struct Finished<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    result: String,
}

impl View for Finished<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![self.result.clone()])
    }
}

pub(crate) fn run(ctx: &Context, args: DoneArgs) -> Result<Report, CliError> {
    let oid = resolve_object(ctx.store.as_ref(), &args.oid)?;
    let plan = plan_complete(ctx.store.as_ref(), &oid)?;
    let force = how_far_this_run_forces(ctx, &args, &plan.blockers)?;
    let outcome = complete(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &oid,
        force,
        tracked_by_a_remote(ctx, &oid)?,
    )?;
    let result = match outcome {
        Completed::Done => "done".to_string(),
        Completed::RolledForward { next_due } => format!("rolled forward to {next_due}"),
    };
    report(&Finished { oid: &oid, result })
}

fn tracked_by_a_remote(ctx: &Context, oid: &Oid) -> Result<Tracked, CliError> {
    for remote in &ctx.config.remotes {
        if ctx.store.remote_id(&remote.name, oid)?.is_some() {
            return Ok(Tracked::ByRemote);
        }
    }
    Ok(Tracked::No)
}

fn how_far_this_run_forces(
    ctx: &Context,
    args: &DoneArgs,
    blockers: &Blockers,
) -> Result<Force, CliError> {
    if !args.force {
        return Ok(Force::No);
    }
    match dispositions_answered_on_the_command_line_over_any_config(args) {
        Some(d) => Ok(Force::With(d)),
        None if args.interactive || ctx.config.done_interactive => {
            Ok(Force::With(ask(ctx, blockers)?))
        }
        None => Ok(Force::With(Dispositions::default())),
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

fn ask(ctx: &Context, blockers: &Blockers) -> Result<Dispositions, CliError> {
    let children = blockers.children.len();
    let deps = blockers.dependencies.len();
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
