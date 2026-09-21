use dam_application::{commit, log};
use dam_domain::CommitRecord;

use crate::args::CommitArgs;
use crate::commands::status::change_json;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run_commit(ctx: &mut Context, args: CommitArgs) -> Result<Report, CliError> {
    let record = commit(
        ctx.store.as_ref(),
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &args.message,
    )?;
    Ok(Report {
        human: commit_line(&record),
        data: commit_json(&record)?,
    })
}

pub(crate) fn run_log(ctx: &mut Context) -> Result<Report, CliError> {
    let records = log(ctx.store.as_ref())?;
    let commits: Vec<serde_json::Value> =
        records.iter().map(commit_json).collect::<Result<_, _>>()?;
    Ok(Report {
        human: records
            .iter()
            .map(commit_line)
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "commits": commits }),
    })
}

pub(crate) fn commit_line(record: &CommitRecord) -> String {
    format!(
        "{}  {}  {} change(s)  {}",
        record.id.short(),
        record.at.strftime("%Y-%m-%d %H:%M"),
        record.changes.len(),
        record.message
    )
}

pub(crate) fn commit_json(record: &CommitRecord) -> Result<serde_json::Value, CliError> {
    let changes: Vec<serde_json::Value> = record
        .changes
        .iter()
        .map(|c| change_json(c, false))
        .collect::<Result<_, _>>()?;
    Ok(serde_json::json!({
        "id": record.id.to_string(),
        "at": record.at.to_string(),
        "message": record.message,
        "changes": changes,
    }))
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
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(
                Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
                "a",
            )))
            .unwrap();
        run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        let first = super::run_commit(
            &mut ctx,
            CommitArgs {
                message: "first".into(),
            },
        )
        .unwrap();
        assert!(first.human.contains("first"));
        assert!(matches!(
            super::run_commit(
                &mut ctx,
                CommitArgs {
                    message: "empty".into()
                }
            ),
            Err(CliError::UseCase(_))
        ));
        let log = super::run_log(&mut ctx).unwrap();
        assert!(log.human.lines().next().unwrap().contains("first"));
        assert_eq!(log.data["commits"][0]["message"], "first");
        assert_eq!(
            log.data["commits"][0]["changes"].as_array().unwrap().len(),
            1
        );
    }
}
