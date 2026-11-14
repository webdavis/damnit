use super::NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM;
use dam_application::{Notice, NoticeRepository, RemoteName};

#[test]
fn a_notice_table_that_is_never_cleared_keeps_the_newest_and_drops_the_oldest() {
    let store = crate::sqlite::SqliteStore::in_memory().unwrap();
    for n in 0..NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM + 5 {
        store
            .add_notice(&Notice::PullFailed {
                remote: RemoteName("t".into()),
                why: format!("reason {n}"),
            })
            .unwrap();
    }
    let kept = store.notices().unwrap();
    assert_eq!(
        kept.len() as i64,
        NOTICES_KEPT_FOR_AN_OPERATOR_WHO_NEVER_CLEARS_THEM
    );
    let oldest = match &kept[0] {
        Notice::PullFailed { why, .. } => why.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(oldest, "reason 5", "the five oldest went");
}
