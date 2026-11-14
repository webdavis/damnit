use dam_domain::{Oid, Path};

use super::order::{BOTH_OID_ORDERS, held_upstream};
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, config, oid, pushing_to_a_remote_that_removes_only_empty_containers,
    remotes_launched_by, task_in, todoist,
};

use MutationOp::Update;

const NAMESAKE: u8 = 3;

fn refusing(refused: Oid) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: if m.oid == refused {
                    MutationVerdict::Refused {
                        why: "temporary failure".into(),
                        unasked: false,
                    }
                } else {
                    MutationVerdict::Accepted { remote_id: None }
                },
            })
            .collect()
    }
}

fn sent_to_a_remote_that_removes_only_empty_containers(
    s: &Scenario,
    refused: Oid,
) -> Vec<(Oid, MutationOp)> {
    let (l, pushed) = pushing_to_a_remote_that_removes_only_empty_containers(refusing(refused));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect()
}

fn assert_held(s: &Scenario, held: u8) {
    assert!(
        s.store
            .push_retries(&todoist())
            .unwrap()
            .contains(&oid(held))
    );
    assert!(s.store.remote_id(&todoist(), &oid(held)).unwrap().is_some());
}

#[test]
fn a_removal_held_by_a_refused_move_stays_held_beside_a_namesake() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(NAMESAKE, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.filed(task_in(child, "c", "Home/")).deleted(&oid(parent));
        assert_eq!(
            sent_to_a_remote_that_removes_only_empty_containers(&s, oid(child)),
            vec![(oid(child), Update)]
        );
        assert_held(&s, parent);
    }
}

#[test]
fn a_removal_held_by_a_child_outside_the_remote_path_stays_held_beside_a_namesake() {
    for (parent, child) in BOTH_OID_ORDERS {
        let mut scoped = config();
        scoped.remotes[0].path = Some(Path::parse("Home").unwrap());
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(NAMESAKE, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ])
        .configured(scoped);
        s.filed(task_in(child, "c", "Work/")).deleted(&oid(parent));
        assert!(sent_to_a_remote_that_removes_only_empty_containers(&s, oid(0)).is_empty());
        assert_held(&s, parent);
    }
}
