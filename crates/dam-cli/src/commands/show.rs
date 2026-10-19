use dam_application::{Refusal, UseCaseError, resolve_commit, resolve_object};
use dam_domain::Object;

use crate::args::ShowArgs;
use crate::commands::commit::{commit_json, commit_line};
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::commands::status::change_line;
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, object_json, object_line};

pub(crate) fn run_show(ctx: &mut Context, args: ShowArgs) -> Result<Report, CliError> {
    refresh_stale_unless_no_pull(ctx)?;
    if let Some(object) = working_object_named(ctx, &args.id)? {
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
            data: object_json(&object)?,
        });
    }
    let record = resolve_commit(ctx.store.as_ref(), &args.id)?;
    let mut human = vec![commit_line(&record)];
    human.extend(
        record
            .changes
            .iter()
            .map(|c| format!("  {}", change_line(c))),
    );
    Ok(Report {
        human: human.join("\n"),
        data: commit_json(&record)?,
    })
}

fn working_object_named(ctx: &Context, text: &str) -> Result<Option<Object>, CliError> {
    match resolve_object(ctx.store.as_ref(), text) {
        Ok(oid) => Ok(ctx.store.get(&oid)?),
        Err(UseCaseError::Refused(Refusal::NoWorkingObject(_))) => Ok(None),
        Err(e) => Err(e.into()),
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
        let oid = Oid::generate(&mut |x: &mut [u8]| {
            x.fill(A_BYTE_APART_FROM_THE_COMMIT_ID_COUNTING_RANDOM_FILLS_FIRST)
        });
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

    const A_BYTE_APART_FROM_THE_COMMIT_ID_COUNTING_RANDOM_FILLS_FIRST: u8 = 3;

    struct TwinPrefixRandom(std::cell::Cell<u8>);
    impl dam_application::Randomness for TwinPrefixRandom {
        fn fill(&self, buf: &mut [u8]) {
            buf.fill(0xab);
            let len = buf.len();
            buf[len - 1] = self.0.get();
            self.0.set(self.0.get().wrapping_add(1));
        }
    }

    #[test]
    fn an_ambiguous_commit_prefix_names_every_match() {
        let mut ctx = context();
        ctx.random = Box::new(TwinPrefixRandom(std::cell::Cell::new(1)));
        ctx.store
            .put(&Object::Task(Task::new(
                Oid::generate(&mut |x: &mut [u8]| x.fill(9)),
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
        run_commit(
            &mut ctx,
            CommitArgs {
                message: "first".into(),
            },
        )
        .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(
                Oid::generate(&mut |x: &mut [u8]| x.fill(10)),
                "b",
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
        run_commit(
            &mut ctx,
            CommitArgs {
                message: "second".into(),
            },
        )
        .unwrap();
        let err = super::run_show(
            &mut ctx,
            ShowArgs {
                id: "abababab".into(),
            },
        )
        .unwrap_err();
        let ids: Vec<String> = ctx
            .store
            .log()
            .unwrap()
            .iter()
            .map(|c| c.id.to_string())
            .collect();
        assert_eq!(err.exit_code(), 2);
        assert_eq!(err.document()["error"]["rule"], "ambiguous_prefix");
        assert_eq!(err.document()["error"]["oids"], serde_json::json!(ids));
    }
}
