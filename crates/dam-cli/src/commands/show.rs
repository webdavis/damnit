use dam_application::{Refusal, UseCaseError, log};

use crate::args::ShowArgs;
use crate::commands::commit::{commit_json, commit_line};
use crate::commands::remote::maybe_pull_stale;
use crate::commands::status::change_line;
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::{Report, object_json, object_line};

pub fn run_show(ctx: &mut Context, args: ShowArgs) -> Result<Report, CliError> {
    maybe_pull_stale(ctx)?;
    if let Ok(oid) = resolve_oid(ctx.store.as_ref(), &args.id)
        && let Some(object) = ctx.store.get(&oid)?
    {
        let b = object.base();
        let mut human = vec![object_line(&object), format!("oid: {}", b.oid)];
        if !b.body.is_empty() {
            human.push(String::new());
            human.push(b.body.clone());
        }
        if !b.depends.is_empty() {
            human.push(format!(
                "depends: {}",
                b.depends
                    .iter()
                    .map(|d| d.short())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        return Ok(Report {
            human: human.join("\n"),
            data: object_json(&object),
        });
    }
    let matches: Vec<_> = log(ctx.store.as_ref())?
        .into_iter()
        .filter(|c| c.id.as_str().starts_with(&args.id))
        .collect();
    match matches.as_slice() {
        [record] => {
            let mut human = vec![commit_line(record)];
            human.extend(
                record
                    .changes
                    .iter()
                    .map(|c| format!("  {}", change_line(c))),
            );
            Ok(Report {
                human: human.join("\n"),
                data: commit_json(record),
            })
        }
        [] => Err(UseCaseError::Refused(Refusal::NoSuchObject(args.id)).into()),
        many => Err(CliError::Usage(format!(
            "{} matches {} commits",
            args.id,
            many.len()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::args::{AddArgs, CommitArgs, ShowArgs};
    use crate::commands::commit::run_commit;
    use crate::commands::stage::run_add;
    use crate::error::CliError;
    use crate::testing::context;
    use dam_domain::{Object, Oid, Task};

    #[test]
    fn show_finds_an_object_or_a_commit_by_prefix() {
        let mut ctx = context();
        // Byte 1 collides with CountingRandom's first fill (0 -> 1), which
        // would make this object's oid equal the commit id it produces below.
        let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(3));
        ctx.store
            .put(&Object::Task(Task::new(oid.clone(), "milk")))
            .unwrap();
        run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        run_commit(
            &mut ctx,
            CommitArgs {
                message: "first".into(),
            },
        )
        .unwrap();
        let object = super::run_show(
            &mut ctx,
            ShowArgs {
                id: oid.short().into(),
            },
        )
        .unwrap();
        assert_eq!(object.data["subject"], "milk");
        assert!(object.human.contains("milk"));
        let id = ctx.store.log().unwrap()[0].id.to_string();
        let commit = super::run_show(&mut ctx, ShowArgs { id: id[..8].into() }).unwrap();
        assert_eq!(commit.data["message"], "first");
        assert!(matches!(
            super::run_show(&mut ctx, ShowArgs { id: "ffff".into() }),
            Err(CliError::UseCase(_))
        ));
    }
}
