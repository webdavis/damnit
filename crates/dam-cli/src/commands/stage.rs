use dam_application::{add, add_all, reset};
use dam_domain::Oid;

use crate::args::{AddArgs, ResetArgs};
use crate::commands::status::{change_json, change_line};
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::Report;

pub(crate) fn run_add(ctx: &mut Context, args: AddArgs) -> Result<Report, CliError> {
    let changes = if args.all {
        add_all(ctx.store.as_ref(), ctx.store.as_ref(), ctx.store.as_ref())?
    } else {
        let oids = args
            .oids
            .iter()
            .map(|t| resolve_oid(ctx.store.as_ref(), t))
            .collect::<Result<Vec<Oid>, _>>()?;
        add(ctx.store.as_ref(), ctx.store.as_ref(), &oids)?
    };
    Ok(Report {
        human: changes
            .iter()
            .map(change_line)
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "staged": changes.iter().map(change_json).collect::<Vec<_>>() }),
    })
}

pub(crate) fn run_reset(ctx: &mut Context, args: ResetArgs) -> Result<Report, CliError> {
    let oids = args
        .oids
        .iter()
        .map(|t| resolve_oid(ctx.store.as_ref(), t))
        .collect::<Result<Vec<Oid>, _>>()?;
    reset(ctx.store.as_ref(), &oids)?;
    let human = if oids.is_empty() {
        "unstaged everything".to_string()
    } else {
        format!(
            "unstaged {}",
            oids.iter().map(Oid::short).collect::<Vec<_>>().join(", ")
        )
    };
    Ok(Report {
        data: serde_json::json!({ "unstaged": oids.iter().map(ToString::to_string).collect::<Vec<_>>() }),
        human,
    })
}

#[cfg(test)]
mod tests {
    use crate::args::{AddArgs, ResetArgs};
    use crate::testing::context;
    use dam_domain::{Object, Oid, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    #[test]
    fn add_stages_named_objects_and_all_stages_everything() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "a")))
            .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(oid(2), "b")))
            .unwrap();
        let report = super::run_add(
            &mut ctx,
            AddArgs {
                oids: vec![oid(1).as_str().into()],
                all: false,
            },
        )
        .unwrap();
        assert_eq!(ctx.store.staged().unwrap().len(), 1);
        assert!(report.human.contains("a"));
        super::run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        assert_eq!(ctx.store.staged().unwrap().len(), 2);
    }

    #[test]
    fn reset_unstages_one_or_all() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "a")))
            .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(oid(2), "b")))
            .unwrap();
        super::run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        super::run_reset(
            &mut ctx,
            ResetArgs {
                oids: vec![oid(1).as_str().into()],
            },
        )
        .unwrap();
        assert_eq!(ctx.store.staged().unwrap().len(), 1);
        super::run_reset(&mut ctx, ResetArgs { oids: vec![] }).unwrap();
        assert!(ctx.store.staged().unwrap().is_empty());
    }
}
