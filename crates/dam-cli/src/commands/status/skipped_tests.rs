use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{Notice, RejectedObject, RemoteName};

fn status_after(notice: Notice) -> crate::output::Report {
    let ctx = context();
    ctx.store.add_notice(&notice).unwrap();
    super::run_status(&ctx, StatusArgs { full: false }).unwrap()
}

fn skipped(remote_id: &str, why: &str) -> Notice {
    Notice::PullSkipped {
        remote: RemoteName("fake".into()),
        object: RejectedObject {
            remote_id: remote_id.into(),
            why: why.into(),
        },
    }
}

#[test]
fn a_skipped_object_with_no_remote_id_reads_as_an_object_skipped() {
    let human = status_after(skipped("", "it has no remote id")).human;
    assert!(
        human.contains("pull from fake skipped an object: it has no remote id"),
        "{human}"
    );
}

#[test]
fn an_object_a_pull_skipped_reads_as_skipped_not_as_a_failed_pull() {
    let report = status_after(skipped("up-7", "its parent chain loops"));
    assert!(
        report
            .human
            .contains("pull from fake skipped up-7: its parent chain loops"),
        "{}",
        report.human
    );
    assert!(!report.human.contains("failed"), "{}", report.human);
    assert_eq!(
        report.data["notices"],
        serde_json::json!([{
            "kind": "pull_skipped",
            "remote": "fake",
            "remote_id": "up-7",
            "why": "its parent chain loops"
        }])
    );
}
