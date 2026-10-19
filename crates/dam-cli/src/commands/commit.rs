use dam_domain::{CommitId, CommitRecord, Timestamp};
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::args::CommitArgs;
use crate::commands::status::ChangeRow;
use crate::context::Context;
use crate::error::CliError;
use crate::layout::counted;
use crate::output::{Report, local_time};
use crate::view::{Human, View, display, report};

pub(crate) fn run_commit(ctx: &Context, args: CommitArgs) -> Result<Report, CliError> {
    let record = ctx.dam().commit(&args.message)?;
    report(&CommitDoc::new(&record, &ctx.tz))
}

pub(crate) fn run_log(ctx: &Context) -> Result<Report, CliError> {
    let records = ctx.store.log()?;
    report(&Log {
        commits: records.iter().map(|r| CommitDoc::new(r, &ctx.tz)).collect(),
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
}

impl<'a> CommitDoc<'a> {
    pub(crate) fn new(record: &'a CommitRecord, zone: &'a TimeZone) -> CommitDoc<'a> {
        CommitDoc {
            id: &record.id,
            at: &record.at,
            message: &record.message,
            changes: record
                .changes
                .iter()
                .map(|c| ChangeRow::new(c, false))
                .collect(),
            zone,
        }
    }

    pub(crate) fn cells(&self) -> Vec<String> {
        vec![
            self.id.short().to_string(),
            local_time(*self.at, self.zone),
            counted(self.changes.len(), "change", "changes"),
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
    use dam_application::Clock;
    use dam_domain::{Object, Oid, Task, Timestamp};
    use jiff::civil::Date;

    struct TwoPmUtc;
    impl Clock for TwoPmUtc {
        fn today(&self) -> Date {
            self.now().to_zoned(jiff::tz::TimeZone::UTC).date()
        }
        fn now(&self) -> Timestamp {
            "2026-10-13T14:00:00Z".parse().unwrap()
        }
    }

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

    #[test]
    fn log_prints_commit_times_in_local_time_like_ls() {
        let mut ctx = context();
        ctx.clock = Box::new(TwoPmUtc);
        ctx.tz = jiff::tz::TimeZone::get("America/New_York").unwrap();
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
        super::run_commit(
            &ctx,
            CommitArgs {
                message: "standup notes".into(),
            },
        )
        .unwrap();
        let log = super::run_log(&ctx).unwrap();
        let id = log.data["commits"][0]["id"].as_str().unwrap()[..7].to_string();
        assert_eq!(
            log.human,
            format!("{id}  2026-10-13 10:00  1 change  standup notes")
        );
        assert_eq!(log.data["commits"][0]["at"], "2026-10-13T14:00:00Z");
    }
}
