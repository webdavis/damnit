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
