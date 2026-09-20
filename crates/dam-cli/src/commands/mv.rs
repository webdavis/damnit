use dam_application::relocate;
use dam_domain::Path;

use crate::args::MvArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::{Report, object_json, object_line};

pub(crate) fn run(ctx: &mut Context, args: MvArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let to =
        Path::parse(&args.path).map_err(|e| CliError::Usage(format!("{}: {e:?}", args.path)))?;
    relocate(ctx.store.as_ref(), &oid, &to)?;
    let object = ctx
        .store
        .get(&oid)?
        .ok_or_else(|| CliError::Io("the moved object did not persist".into()))?;
    Ok(Report {
        human: object_line(&object),
        data: object_json(&object)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::MvArgs;
    use crate::testing::context;
    use dam_domain::{Object, Oid, Task};

    #[test]
    fn mv_relocates() {
        let mut ctx = context();
        let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
        ctx.store
            .put(&Object::Task(Task::new(oid.clone(), "x")))
            .unwrap();
        run(
            &mut ctx,
            MvArgs {
                oid: oid.as_str().into(),
                path: "work/".into(),
            },
        )
        .unwrap();
        assert_eq!(
            ctx.store.get(&oid).unwrap().unwrap().base().path.as_str(),
            "work/"
        );
    }
}
