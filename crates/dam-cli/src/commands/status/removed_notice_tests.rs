use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{Notice, RemoteName};
use dam_domain::Oid;

#[test]
fn a_task_removed_in_the_app_is_noticed_as_kept_here_with_both_ways_out() {
    let ctx = context();
    let oid = Oid::generate(&mut |b: &mut [u8]| b.fill(0x1a));
    ctx.store
        .add_notice(&Notice::RemovedUpstream {
            remote: RemoteName("fake".into()),
            oid: oid.clone(),
            subject: "old errand".into(),
        })
        .unwrap();
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    let short = oid.short();
    let line = format!(
        "{short} was removed on fake: \"old errand\" is kept here; dam rm {short} to drop it, \
         or edit it to send it to fake again"
    );
    assert!(report.human.contains(&line), "{}", report.human);
}
