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
