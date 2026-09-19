use dam_application::remove;
use dam_domain::Oid;

use crate::args::RmArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::Report;

pub fn run(ctx: &mut Context, args: RmArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let plan = remove(ctx.store.as_ref(), &oid)?;
    let shorts = |v: &[Oid]| v.iter().map(Oid::short).collect::<Vec<_>>().join(", ");
    let mut human = format!("removed {}", oid.short());
    if !plan.descendants.is_empty() {
        human.push_str(&format!("\nmoved up: {}", shorts(&plan.descendants)));
    }
    if !plan.dependents.is_empty() {
        human.push_str(&format!(
            "\nno longer depend on it: {}",
            shorts(&plan.dependents)
        ));
    }
    if !plan.attached.is_empty() {
        human.push_str(&format!(
            "\nstill attached to it: {}",
            shorts(&plan.attached)
        ));
    }
    let ids = |v: &[Oid]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
    Ok(Report {
        human,
        data: serde_json::json!({
            "oid": oid.to_string(),
            "descendants": ids(&plan.descendants),
            "dependents": ids(&plan.dependents),
            "attached": ids(&plan.attached),
        }),
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
        let mut ctx = context();
        let mut p = Task::new(oid(1), "p");
        p.base.path = Path::parse("p").unwrap();
        let mut c = Task::new(oid(2), "c");
        c.base.path = Path::parse("p/c").unwrap();
        ctx.store.put(&Object::Task(p)).unwrap();
        ctx.store.put(&Object::Task(c)).unwrap();
        let report = run(
            &mut ctx,
            RmArgs {
                oid: oid(1).as_str().into(),
            },
        )
        .unwrap();
        assert!(ctx.store.get(&oid(1)).unwrap().is_none());
        assert!(report.human.contains(oid(2).short()));
        assert_eq!(report.data["descendants"].as_array().unwrap().len(), 1);
    }
}
