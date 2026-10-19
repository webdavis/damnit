use dam_domain::Kind;

use crate::testing::prelude::*;
use crate::testing::{Scenario, pushing, remotes_launched_by, todoist};

#[test]
fn a_push_records_the_kinds_the_remote_accepts() {
    let s = Scenario::default();
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store.accepted_kinds(&todoist()).unwrap(),
        Some(vec![Kind::Task])
    );
}
