use crate::ports::HelperError;
use crate::remote::PullOutcome;
use crate::testing::{Said, Scenario, answering, pulling, remotes_launched_by};

fn pulling_from_todoist() -> Said {
    Said::Started("pull@todoist: pulling from todoist ".into(), None)
}

#[test]
fn the_wait_for_a_remote_is_a_step_named_after_the_remote_that_finishes() {
    let s = Scenario::default();
    let l = pulling(PullOutcome::default());
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [pulling_from_todoist(), Said::Finished]
    );
}

#[test]
fn a_remote_that_fails_to_answer_ends_the_step_as_failed() {
    let s = Scenario::default();
    let l = answering(Err(HelperError::Cancelled));
    s.dam().pull(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [pulling_from_todoist(), Said::Failed]
    );
}
