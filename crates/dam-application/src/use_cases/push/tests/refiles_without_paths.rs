use dam_domain::Field;

use super::refiles::{a_parent_replaced_by_a_namesake_over_a_child_it_held, sent};
use crate::remote::{MutationOp, RemoteCapabilities};
use crate::testing::{accepting_all, oid, pushing_with, remotes_launched_by, task_caps};

#[test]
fn a_remote_that_does_not_file_by_path_is_sent_no_refile_for_a_child_whose_parent_was_replaced() {
    let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(1, 2, 3);
    let caps = RemoteCapabilities {
        fields: task_caps()
            .fields
            .into_iter()
            .filter(|f| *f != Field::Path)
            .collect(),
        removes_only_empty_containers: true,
        ..task_caps()
    };
    let (l, pushed) = pushing_with(caps, accepting_all(Some("i:new")));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    assert_eq!(report.skipped, 0);
    let ops: Vec<_> = sent(&pushed)
        .into_iter()
        .map(|(o, op, _, _)| (o, op))
        .collect();
    assert_eq!(ops, vec![(oid(2), MutationOp::Create)]);
}
