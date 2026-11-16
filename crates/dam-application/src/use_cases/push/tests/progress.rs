use crate::ports::HelperError;
use crate::remote::PullOutcome;
use crate::testing::{
    Said, Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, pushing, remotes_launched_by,
    task,
};

fn sending_to_todoist(total: usize) -> Said {
    Said::Started(
        "push@todoist: sending to todoist changes".into(),
        Some(total),
    )
}

#[test]
fn the_changes_sent_to_a_remote_are_a_step_counted_of_the_total_that_finishes() {
    let s = Scenario::default();
    s.committed(task(1));
    s.committed(task(2));
    let (l, _) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [sending_to_todoist(2), Said::Advanced(2), Said::Finished]
    );
}

#[test]
fn a_remote_that_stops_the_push_ends_the_step_as_failed() {
    let s = Scenario::default();
    s.committed(task(1));
    let l = ScriptedLauncher::launching(|| ScriptedHelper {
        push_answer: Box::new(|_| Err(HelperError::Remote("rate limited".into()))),
        ..ScriptedHelper::answering(Ok(PullOutcome::default()))
    });
    let _ = s.dam().push(remotes_launched_by(&l), None);
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [sending_to_todoist(1), Said::Advanced(1), Said::Failed]
    );
}

#[test]
fn nothing_to_send_shows_no_step() {
    let s = Scenario::default();
    let (l, _) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(*s.fixture.progress.said.borrow(), []);
}
