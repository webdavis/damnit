use dam_application::{relocate, resolve_object};

use crate::args::MvArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectRow, report};

pub(crate) fn run(ctx: &Context, args: MvArgs) -> Result<Report, CliError> {
    let oid = resolve_object(ctx.store.as_ref(), &args.oid)?;
    relocate(ctx.store.as_ref(), &oid, &args.path)?;
    let object = ctx
        .store
        .get(&oid)?
        .ok_or_else(|| CliError::Internal("the moved object did not persist".into()))?;
    report(&ObjectRow(&object, &ctx.tz))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::MvArgs;
    use crate::testing::context;
    use dam_domain::{Object, Oid, Path, Task};

    #[test]
    fn mv_relocates() {
        let ctx = context();
        let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
        ctx.store
            .put(&Object::Task(Task::new(oid.clone(), "x")))
            .unwrap();
        run(
            &ctx,
            MvArgs {
                oid: oid.as_str().into(),
                path: Path::parse("work/").unwrap(),
            },
        )
        .unwrap();
        assert_eq!(
            ctx.store.get(&oid).unwrap().unwrap().base().path.as_str(),
            "work/"
        );
    }
}
