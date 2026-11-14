use dam_domain::Oid;
use serde::Serialize;

use dam_application::{Named, Removal};

use self::answer::Answer;
use crate::args::RmArgs;
use crate::commands::follow_ons::FollowOns;
use crate::commands::settled::Settled;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, report};

#[derive(Serialize)]
struct Removed<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    #[serde(flatten)]
    follow_ons: FollowOns<'a>,
}

impl View for Removed<'_> {
    fn human(&self) -> Human {
        let mut lines = vec![format!("removed {}", self.oid.short())];
        lines.extend(self.follow_ons.lines());
        Human::Lines(lines)
    }
}

pub(crate) fn run(ctx: &Context, args: RmArgs) -> Result<Report, CliError> {
    let Some(named) = args.oid else {
        let id = args
            .id
            .ok_or_else(|| CliError::Usage("rm takes an oid or --id".into()))?;
        return report_removal(&ctx.dam().remove_by_id(&id)?);
    };
    let removal = match ctx.dam().resolve_named(&named)? {
        Named::Object(oid) => ctx.dam().remove_or_send(&oid)?,
        Named::Item(hash) => ctx.dam().remove_item(&hash)?,
    };
    report_removal(&removal)
}

fn report_removal(removal: &Removal) -> Result<Report, CliError> {
    match removal {
        Removal::Removed(plan) => report(&Removed {
            oid: &plan.oid,
            follow_ons: FollowOns::of(plan),
        }),
        Removal::ItemDeclined(item) => report(&Settled::new(item)),
        Removal::Withdrawn(about) => report(&Answer::about("withdrawn", about)),
        Removal::Waiting(about) => report(&Answer::about("waiting", about)),
        Removal::Declined(about) => report(&Answer::about("declined", about)),
        Removal::Nothing => report(&Answer::nothing()),
    }
}

mod answer;

#[cfg(test)]
mod tests;
