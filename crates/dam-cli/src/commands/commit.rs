use dam_domain::{CommitId, CommitRecord, Timestamp};
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::args::CommitArgs;
use crate::commands::status::{ChangeRow, the_owner_to};
use crate::context::Context;
use crate::error::CliError;
use crate::layout::counted;
use crate::output::{Report, local_time};
use crate::view::{Human, View, display, report};

pub(crate) fn run_commit(ctx: &Context, args: CommitArgs) -> Result<Report, CliError> {
    let record = ctx.dam().commit(&args.message)?;
    report(&CommitDoc::new(&record, ctx))
}

pub(crate) fn run_log(ctx: &Context) -> Result<Report, CliError> {
    let records = ctx.store.log()?;
    report(&Log {
        commits: records.iter().map(|r| CommitDoc::new(r, ctx)).collect(),
    })
}

#[derive(Serialize)]
struct Log<'a> {
    commits: Vec<CommitDoc<'a>>,
}

impl View for Log<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.commits.iter().map(CommitDoc::cells).collect(),
            "no commits yet",
        )
    }
}

#[derive(Serialize)]
pub(crate) struct CommitDoc<'a> {
    #[serde(serialize_with = "display")]
    id: &'a CommitId,
    #[serde(serialize_with = "display")]
    at: &'a Timestamp,
    message: &'a str,
    pub(crate) changes: Vec<ChangeRow<'a>>,
    #[serde(skip)]
    zone: &'a TimeZone,
    #[serde(skip)]
    owner: &'static str,
}

impl<'a> CommitDoc<'a> {
    pub(crate) fn new(record: &'a CommitRecord, ctx: &'a Context) -> CommitDoc<'a> {
        CommitDoc {
            id: &record.id,
            at: &record.at,
            message: &record.message,
            changes: record
                .changes
                .iter()
                .map(|c| ChangeRow::authored(ctx, c, false))
                .collect(),
            zone: &ctx.tz,
            owner: the_owner_to(ctx.access.as_ref()),
        }
    }

    pub(crate) fn cells(&self) -> Vec<String> {
        vec![
            self.id.short().to_string(),
            local_time(*self.at, self.zone),
            counted(self.changes.len(), "change", "changes"),
            self.senders(),
            self.message.to_string(),
        ]
    }

    fn senders(&self) -> String {
        let mut senders: Vec<&str> = Vec::new();
        for sender in self.changes.iter().map(|c| c.sender(self.owner)) {
            if !senders.contains(&sender) {
                senders.push(sender);
            }
        }
        senders.join(", ")
    }
}

impl View for CommitDoc<'_> {
    fn human(&self) -> Human {
        Human::Table(vec![self.cells()])
    }
}

#[cfg(test)]
mod tests;
