mod asked;

use dam_application::{Blockers, ChildDisposition, Completed, Dispositions, Force, Tracked};

use crate::args::DoneArgs;
use crate::commands::each::{one_object, report_set, set_of};
use crate::commands::hooked::hooked;
use crate::commands::sent::report_landed;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, report};
use asked::ask;
use dam_domain::{Oid, Path};
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
    let Some(oid) = one_object(&args.oid, &args.set) else {
        return run_on_a_set(ctx, &args);
    };
    let oid = ctx.dam().resolve_object(oid)?;
    let plan = ctx.dam().plan_complete(&oid)?;
    let rule = the_projects_rule_for(&args, &plan.path);
    let force = how_far_this_run_forces(ctx, &args, rule, &plan.blockers)?;
    let (outcome, landed) = ctx
        .dam()
        .complete(&oid, force, tracked_by_a_remote(ctx, &oid)?)?;
    let result = match outcome {
        Completed::Done => "done".to_string(),
        Completed::RolledForward { next_due } => format!("rolled forward to {next_due}"),
    };
    report_landed(&landed, |_| {
        report(&hooked(ctx, Finished { oid: &oid, result }))
    })
}

fn run_on_a_set(ctx: &Context, args: &DoneArgs) -> Result<Report, CliError> {
    if args.interactive {
        return Err(CliError::Usage(
            "done --interactive takes one oid; name what happens with --children and --depends instead".into(),
        ));
    }
    let (selector, run) = set_of(ctx, &args.oid, &args.set)?;
    let force = |oid: &Oid| match args.force {
        true => {
            let path = ctx.dam().plan_complete(oid).map(|plan| plan.path);
            let rule = path
                .ok()
                .and_then(|path| the_projects_rule_for(args, &path));
            Force::With(
                dispositions_answered_on_the_command_line_over_any_config(args, rule)
                    .unwrap_or_default(),
            )
        }
        false => Force::No,
    };
    let tracked = |oid: &Oid| tracked_by_a_remote(ctx, oid).unwrap_or(Tracked::No);
    let changes = ctx.dam().complete_each(&selector, &force, &tracked, run)?;
    report_set(ctx, &changes)
}

fn tracked_by_a_remote(ctx: &Context, oid: &Oid) -> Result<Tracked, CliError> {
    for remote in &ctx.config.remotes {
        if ctx.store.remote_id(&remote.name, oid)?.is_some() {
            return Ok(Tracked::ByRemote);
        }
    }
    Ok(Tracked::No)
}

fn the_projects_rule_for(args: &DoneArgs, path: &Path) -> Option<ChildDisposition> {
    let (place, rule) = args.project_children.as_ref()?;
    path.is_within(place).then(|| rule.clone())
}

fn how_far_this_run_forces(
    ctx: &Context,
    args: &DoneArgs,
    rule: Option<ChildDisposition>,
    blockers: &Blockers,
) -> Result<Force, CliError> {
    if !args.force {
        return Ok(Force::No);
    }
    if args.interactive {
        return Ok(Force::With(ask(ctx, blockers)?));
    }
    match dispositions_answered_on_the_command_line_over_any_config(args, rule) {
        Some(d) => Ok(Force::With(d)),
        None if ctx.config.done_interactive() => Ok(Force::With(ask(ctx, blockers)?)),
        None => Ok(Force::With(Dispositions::default())),
    }
}

fn dispositions_answered_on_the_command_line_over_any_config(
    args: &DoneArgs,
    rule: Option<ChildDisposition>,
) -> Option<Dispositions> {
    match (args.children.clone().or(rule), args.depends) {
        (None, None) => None,
        (children, dependencies) => Some(Dispositions {
            children: children.unwrap_or_default(),
            dependencies: dependencies.unwrap_or_default(),
        }),
    }
}

#[cfg(test)]
mod tests;
