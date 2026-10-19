use dam_domain::Oid;
use serde::Serialize;

use crate::args::RmArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, displays, report};

#[derive(Serialize)]
struct Removed<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    #[serde(serialize_with = "displays")]
    descendants: &'a [Oid],
    #[serde(serialize_with = "displays")]
    dependents: &'a [Oid],
    #[serde(serialize_with = "displays")]
    attached: &'a [Oid],
}

impl View for Removed<'_> {
    fn human(&self) -> Human {
        let shorts = |v: &[Oid]| v.iter().map(Oid::short).collect::<Vec<_>>().join(", ");
        let mut lines = vec![format!("removed {}", self.oid.short())];
        let mut follow = |title: &str, oids: &[Oid]| {
            if !oids.is_empty() {
                lines.push(format!("{title}: {}", shorts(oids)));
            }
        };
        follow("moved up", self.descendants);
        follow("no longer depend on it", self.dependents);
        follow("still attached to it", self.attached);
        Human::Lines(lines)
    }
}

pub(crate) fn run(ctx: &Context, args: RmArgs) -> Result<Report, CliError> {
    let oid = ctx.dam().resolve_object(&args.oid)?;
    let plan = ctx.dam().remove(&oid)?;
    report(&Removed {
        oid: &oid,
        descendants: &plan.descendants,
        dependents: &plan.dependents,
        attached: &plan.attached,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::RmArgs;
    use crate::testing::context;
    use dam_domain::{Object, Oid, Path, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    #[test]
    fn removing_a_parent_moves_its_child_up_and_says_so() {
        let ctx = context();
        let mut p = Task::new(oid(1), "plan");
        p.base.path = Path::parse("work").unwrap();
        let mut c = Task::new(oid(2), "step");
        c.base.path = Path::parse("work/plan").unwrap();
        ctx.store.put(&Object::Task(p)).unwrap();
        ctx.store.put(&Object::Task(c)).unwrap();
        let report = run(
            &ctx,
            RmArgs {
                oid: oid(1).as_str().into(),
            },
        )
        .unwrap();
        assert!(ctx.store.get(&oid(1)).unwrap().is_none());
        assert_eq!(
            ctx.store
                .get(&oid(2))
                .unwrap()
                .unwrap()
                .base()
                .path
                .as_str(),
            "work/"
        );
        assert!(report.human.contains(oid(2).short()));
        assert_eq!(report.data["descendants"].as_array().unwrap().len(), 1);
    }
}
