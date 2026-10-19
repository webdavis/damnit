use super::super::*;
use crate::ports::HelperError;
use crate::testing::prelude::*;
use crate::testing::{Scenario, answering, remotes_launched_by, task_caps, todoist};

#[test]
fn a_helper_that_times_out_leaves_the_store_as_it_was() {
    let s = Scenario::default();
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    let before = s.store.clone();
    let timeout = HelperError::Timeout {
        helper: "todoist".into(),
        deadline: "30s".into(),
        said: None,
    };
    let l = answering(Err(timeout.clone()));
    let err = s.dam().pull(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(err, UseCaseError::Helper(timeout));
    assert_eq!(s.store, before);
}
