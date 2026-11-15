use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{Conflict, RemoteName};
use dam_domain::{Object, Oid, Task};

#[test]
fn a_conflict_over_a_removal_here_shows_ours_as_removed_and_names_the_whole_oid() {
    let ctx = context();
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
    ctx.store
        .mark_conflict(&Conflict {
            remote: RemoteName("fake".into()),
            oid: oid.clone(),
            ours: None,
            theirs: Object::Task(Task::new(oid.clone(), "Ann renamed")),
        })
        .unwrap();
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    assert!(report.human.contains("ours: removed"), "{}", report.human);
    let hint = format!("dam resolve {oid} --ours|--theirs");
    assert!(report.human.contains(&hint), "{}", report.human);
    let held = &report.data["conflicts"][0];
    assert_eq!(held["ours"], serde_json::Value::Null);
    assert_eq!(held["theirs"]["subject"], "Ann renamed");
}
