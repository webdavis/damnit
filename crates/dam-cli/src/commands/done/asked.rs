use dam_application::{Blockers, ChildDisposition, DependencyDisposition, Dispositions};

use crate::context::Context;
use crate::error::CliError;
use crate::layout::counted;

pub(super) fn ask(ctx: &Context, blockers: &Blockers) -> Result<Dispositions, CliError> {
    let children = blockers.children.len();
    let deps = blockers.dependencies.len();
    let children = if children == 0 {
        ChildDisposition::Keep
    } else {
        match ctx.prompt.choose(
            &format!(
                "{}. What happens to them?",
                counted(children, "open child task", "open child tasks")
            ),
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
            &format!(
                "{}. What happens to the links?",
                counted(deps, "open dependency", "open dependencies")
            ),
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
