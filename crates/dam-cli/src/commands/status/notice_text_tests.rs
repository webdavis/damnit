use dam_application::{DroppedBecause, FunctionFailure, FunctionFault, Notice, RemoteName};
use dam_domain::{Oid, Path};
use serde_json::json;

use crate::args::StatusArgs;
use crate::output::Report;
use crate::testing::context;

fn status_with(notice: Notice) -> Report {
    let ctx = context();
    ctx.store.add_notice(&notice).unwrap();
    super::run_status(&ctx, StatusArgs { full: false }).unwrap()
}

#[test]
fn a_failed_pulled_hook_is_shown_with_its_counts_and_first_error() {
    let report = status_with(Notice::TaskPulledFailed {
        remote: RemoteName("fake".into()),
        failed: 1,
        not_reached: 0,
        error: Some(FunctionFailure {
            file: "/home/config.lua".into(),
            line: Some(3),
            fault: FunctionFault::Raised {
                message: "no bell".into(),
            },
        }),
    });
    assert!(
        report.human.contains(
            "task_pulled on the last pull from fake: 1 failed, 0 not reached; \
             first error at /home/config.lua:3: no bell"
        ),
        "{}",
        report.human
    );
    assert_eq!(
        report.data["notices"],
        json!([{"kind": "task_pulled_failed", "remote": "fake", "failed": 1, "not_reached": 0,
                "error": {"file": "/home/config.lua", "line": 3, "message": "no bell"}}])
    );
}

#[test]
fn an_unanswered_create_removed_here_is_shown_with_where_it_was_sent() {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(5));
    let report = status_with(Notice::UnansweredCreateDropped {
        remote: RemoteName("fake".into()),
        oid: oid.clone(),
        subject: "call Ada".into(),
        path: Path::parse("work/").unwrap(),
        because: DroppedBecause::Removed,
    });
    assert!(
        report.human.contains(
            "the create of \"call Ada\" in work/ got no answer from fake, and it was then \
             removed here; fake may hold a copy to remove by hand"
        ),
        "{}",
        report.human
    );
    assert_eq!(
        report.data["notices"],
        json!([{"kind": "unanswered_create_dropped", "remote": "fake", "oid": oid.to_string(),
                "subject": "call Ada", "path": "work/", "because": "removed"}])
    );
}
