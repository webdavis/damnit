use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{Notice, RejectedObject, RemoteName};

fn status_after(notice: Notice) -> String {
    let ctx = context();
    ctx.store.add_notice(&notice).unwrap();
    super::run_status(&ctx, StatusArgs { full: false })
        .unwrap()
        .human
}

#[test]
fn a_skipped_object_with_no_remote_id_reads_as_an_object_skipped() {
    let human = status_after(Notice::PullSkipped {
        remote: RemoteName("fake".into()),
        object: RejectedObject {
            remote_id: String::new(),
            why: "it has no remote id".into(),
        },
    });
    assert!(
        human.contains("pull from fake skipped an object: it has no remote id"),
        "{human}"
    );
}

#[test]
fn a_pull_that_failed_still_reads_as_failed() {
    let human = status_after(Notice::PullFailed {
        remote: RemoteName("fake".into()),
        why: "the helper went away".into(),
    });
    assert!(
        human.contains("pull from fake failed: the helper went away"),
        "{human}"
    );
}
