use dam_domain::Kind;

use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{Scenario, pulling, remotes_launched_by, todoist};

#[test]
fn a_pull_records_the_kinds_the_remote_accepts() {
    let s = Scenario::default();
    s.dam()
        .pull(remotes_launched_by(&pulling(PullOutcome::default())), None)
        .unwrap();
    assert_eq!(
        s.store.accepted_kinds(&todoist()).unwrap(),
        Some(vec![Kind::Task])
    );
}
