use dam_domain::{CommitId, CommitRecord, Timestamp};
use serde::Serialize;

use crate::args::CommitArgs;
use crate::commands::status::ChangeRow;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, report};

pub(crate) fn run_commit(ctx: &Context, args: CommitArgs) -> Result<Report, CliError> {
    let record = ctx.dam().commit(&args.message)?;
    report(&CommitDoc::new(&record))
}

pub(crate) fn run_log(ctx: &Context) -> Result<Report, CliError> {
    let records = ctx.store.log()?;
    report(&Log {
        commits: records.iter().map(CommitDoc::new).collect(),
    })
}

#[derive(Serialize)]
struct Log<'a> {
    commits: Vec<CommitDoc<'a>>,
}

impl View for Log<'_> {
    fn human(&self) -> Human {
        Human::Table(self.commits.iter().map(CommitDoc::cells).collect())
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
}

impl<'a> CommitDoc<'a> {
    pub(crate) fn new(record: &'a CommitRecord) -> CommitDoc<'a> {
        CommitDoc {
            id: &record.id,
            at: &record.at,
            message: &record.message,
            changes: record
                .changes
                .iter()
                .map(|c| ChangeRow::new(c, false))
                .collect(),
        }
    }

    pub(crate) fn cells(&self) -> Vec<String> {
        vec![
            self.id.short().to_string(),
            self.at.strftime("%Y-%m-%d %H:%M").to_string(),
            format!("{} change(s)", self.changes.len()),
            self.message.to_string(),
        ]
    }
}

impl View for CommitDoc<'_> {
    fn human(&self) -> Human {
        Human::Table(vec![self.cells()])
    }
}

#[cfg(test)]
mod tests {
    use crate::args::{AddArgs, CommitArgs};
    use crate::commands::stage::run_add;
    use crate::error::CliError;
    use crate::testing::context;
    use dam_domain::{Object, Oid, Task};

    #[test]
    fn commit_records_the_stage_and_log_lists_it_newest_first() {
        let ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(
                Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
                "a",
            )))
            .unwrap();
        run_add(
            &ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        let first = super::run_commit(
            &ctx,
            CommitArgs {
                message: "first".into(),
            },
        )
        .unwrap();
        assert!(first.human.contains("first"));
        assert!(matches!(
            super::run_commit(
                &ctx,
                CommitArgs {
                    message: "empty".into()
                }
            ),
            Err(CliError::UseCase(_))
        ));
        let log = super::run_log(&ctx).unwrap();
        assert!(log.human.lines().next().unwrap().contains("first"));
        assert_eq!(log.data["commits"][0]["message"], "first");
        assert_eq!(
            log.data["commits"][0]["changes"].as_array().unwrap().len(),
            1
        );
    }
}
