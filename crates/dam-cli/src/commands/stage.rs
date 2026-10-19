use dam_domain::Oid;

use crate::args::{AddArgs, ResetArgs};
use crate::commands::status::ChangeRow;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, displays, report};
use serde::Serialize;

#[derive(Serialize)]
struct Staged<'a> {
    staged: Vec<ChangeRow<'a>>,
}

impl View for Staged<'_> {
    fn human(&self) -> Human {
        Human::Lines(self.staged.iter().map(ChangeRow::line).collect())
    }
}

#[derive(Serialize)]
struct Unstaged<'a> {
    #[serde(serialize_with = "displays")]
    unstaged: &'a [Oid],
}

impl View for Unstaged<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![match self.unstaged {
            [] => "unstaged everything".to_string(),
            oids => format!(
                "unstaged {}",
                oids.iter().map(Oid::short).collect::<Vec<_>>().join(", ")
            ),
        }])
    }
}

pub(crate) fn run_add(ctx: &Context, args: AddArgs) -> Result<Report, CliError> {
    let changes = if args.all {
        ctx.dam().add_all()?
    } else {
        let oids = args
            .oids
            .iter()
            .map(|t| ctx.dam().resolve_object(t))
            .collect::<Result<Vec<Oid>, _>>()?;
        ctx.dam().add(&oids)?
    };
    report(&Staged {
        staged: changes.iter().map(|c| ChangeRow::new(c, false)).collect(),
    })
}

pub(crate) fn run_reset(ctx: &Context, args: ResetArgs) -> Result<Report, CliError> {
    let oids = args
        .oids
        .iter()
        .map(|t| ctx.dam().resolve_object(t))
        .collect::<Result<Vec<Oid>, _>>()?;
    ctx.dam().reset(&oids)?;
    report(&Unstaged { unstaged: &oids })
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
        let ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "a")))
            .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(oid(2), "b")))
            .unwrap();
        let report = super::run_add(
            &ctx,
            AddArgs {
                oids: vec![oid(1).as_str().into()],
                all: false,
            },
        )
        .unwrap();
        assert_eq!(ctx.store.staged().unwrap().len(), 1);
        assert!(report.human.contains("a"));
        super::run_add(
            &ctx,
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
        let ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "a")))
            .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(oid(2), "b")))
            .unwrap();
        super::run_add(
            &ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        super::run_reset(
            &ctx,
            ResetArgs {
                oids: vec![oid(1).as_str().into()],
            },
        )
        .unwrap();
        assert_eq!(ctx.store.staged().unwrap().len(), 1);
        super::run_reset(&ctx, ResetArgs { oids: vec![] }).unwrap();
        assert!(ctx.store.staged().unwrap().is_empty());
    }
}
