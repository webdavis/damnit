use dam_domain::Object;

use super::order::{BOTH_OID_ORDERS, held_upstream, sent};
use crate::ports::Notice;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{oid, task_in, todoist};

use MutationOp::Create;

#[test]
fn a_removal_waits_on_a_task_this_push_files_in_its_folder() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[task_in(parent, "P", "Home/")]);
        s.deleted(&oid(parent))
            .committed(Object::Task(task_in(child, "c", "Home/P/")));
        assert_eq!(sent(&s), vec![(oid(child), Create)]);
        let why = s
            .store
            .notices()
            .unwrap()
            .into_iter()
            .find_map(|n| match n {
                Notice::PushFailed { oid: o, why, .. } if o == oid(parent) => Some(why),
                _ => None,
            })
            .expect("a push failed notice for the parent");
        assert!(why.contains(oid(child).short()), "{why}");
        assert!(
            s.store
                .push_retries(&todoist())
                .unwrap()
                .contains(&oid(parent))
        );
        assert!(
            s.store
                .remote_id(&todoist(), &oid(parent))
                .unwrap()
                .is_some()
        );
    }
}
