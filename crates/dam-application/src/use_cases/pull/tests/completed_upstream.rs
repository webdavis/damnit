use std::cell::RefCell;
use std::rc::Rc;

use super::incoming;
use crate::ports::Clock;
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{Scenario, ScriptedHelper, ScriptedLauncher, oid, remotes_launched_by};
use dam_domain::{Completion, Object};

type Asked = Rc<RefCell<Vec<Option<String>>>>;

fn answering(asked: &Asked, outcome: PullOutcome) -> ScriptedLauncher {
    let asked = asked.clone();
    ScriptedLauncher::launching(move || {
        let mut helper = ScriptedHelper::answering(Ok(outcome.clone()));
        helper.pulled_since = asked.clone();
        helper
    })
}

fn water_the_plants(completion: Completion, sync: &str) -> PullOutcome {
    let mut found = incoming("water the plants", "i:1");
    if let Object::Task(task) = &mut found.object {
        task.completion = completion;
    }
    PullOutcome {
        objects: vec![found],
        sync: Some(sync.into()),
        ..PullOutcome::default()
    }
}

fn completed_today(s: &Scenario) -> Vec<String> {
    s.dam()
        .list(Some("completed:today"))
        .unwrap()
        .into_iter()
        .map(|o| o.base().subject.clone())
        .collect()
}

#[test]
fn a_task_completed_in_the_app_is_found_by_completed_today_and_a_replay_changes_nothing() {
    let s = Scenario::default().drawing_from(7);
    let asked = Asked::default();
    let open = answering(&asked, water_the_plants(Completion::Open, "t1"));
    s.dam().pull(remotes_launched_by(&open), None).unwrap();
    assert!(completed_today(&s).is_empty());

    let done = Completion::Done {
        at: Some(s.fixture.clock.now()),
    };
    let completed = answering(&asked, water_the_plants(done.clone(), "t2"));
    s.dam().pull(remotes_launched_by(&completed), None).unwrap();
    assert_eq!(completed_today(&s), ["water the plants"]);

    let replay = answering(&asked, water_the_plants(done, "t2"));
    let reports = s.dam().pull(remotes_launched_by(&replay), None).unwrap();
    assert_eq!(reports[0].unchanged, 1);
    assert_eq!(completed_today(&s), ["water the plants"]);
    assert_eq!(
        s.store.get(&oid(7)).unwrap().unwrap().base().subject,
        "water the plants"
    );
    assert_eq!(
        *asked.borrow(),
        [None, Some("t1".to_string()), Some("t2".to_string())]
    );
}
