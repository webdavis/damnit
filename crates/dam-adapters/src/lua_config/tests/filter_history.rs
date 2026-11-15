use std::cell::Cell;

use dam_application::{FilterFault, Histories, History};
use dam_domain::{Authored, Change, CommitId, CommitRecord, Object, Oid, Timestamp};

use super::filter_support::{FixedHistory, every_field_task, oid, run_filter_with};
use crate::history_document;

fn postponed(task: &Object) -> CommitRecord {
    let mut later = task.clone();
    if let Object::Task(t) = &mut later {
        t.due = None;
    }
    CommitRecord {
        id: CommitId::generate(&mut |b: &mut [u8]| b.fill(9)),
        message: "push it back".into(),
        pulled: false,
        at: Timestamp::UNIX_EPOCH,
        changes: vec![Authored::by_owner(Change::Update {
            before: task.clone(),
            after: later,
        })],
    }
}

#[derive(Default)]
struct CountedHistory {
    asked: Cell<usize>,
}

impl Histories for CountedHistory {
    fn of(&self, _: &Oid) -> History {
        self.asked.set(self.asked.get() + 1);
        History::Recorded(vec![])
    }
}

#[test]
fn dam_history_returns_the_objects_changes_in_the_shape_the_log_prints() {
    let task = every_field_task();
    let commits = vec![postponed(&task)];
    let expected = history_document(&commits, "owner");
    assert_eq!(expected[0]["changes"][0]["fields"][0], "due");
    let kept = run_filter_with(
        "        local h = dam.history(task)\n        local c = h[1].changes[1]\n        \
         return #h == 1 and h[1].message == 'push it back' and c.op == 'update'\n          \
         and c.fields[1] == 'due' and c.author == 'owner' and c.oid == task.oid",
        &[task],
        &FixedHistory(History::Recorded(commits)),
    );
    assert_eq!(kept.unwrap(), [true]);
}

#[test]
fn the_history_is_asked_for_only_when_match_calls_dam_history() {
    let objects = vec![every_field_task(); 3];
    let counted = CountedHistory::default();
    run_filter_with("        return true", &objects, &counted).unwrap();
    assert_eq!(counted.asked.get(), 0);
    run_filter_with(
        "        dam.history(task)\n        dam.history(task)\n        return true",
        &objects,
        &counted,
    )
    .unwrap();
    assert_eq!(counted.asked.get(), 6);
}

#[test]
fn dam_history_reads_only_the_object_match_was_handed() {
    let failure = run_filter_with(
        &format!("        return #dam.history({{ oid = '{}' }}) > 0", oid(7)),
        &[every_field_task()],
        &FixedHistory(History::Recorded(vec![])),
    )
    .unwrap_err();
    assert!(matches!(failure.fault, FilterFault::Raised { .. }));
}

#[test]
fn a_history_withheld_or_unread_raises_only_when_read() {
    for history in [History::Withheld, History::Unread] {
        let fixed = FixedHistory(history);
        let task = [every_field_task()];
        assert_eq!(
            run_filter_with("        return true", &task, &fixed).unwrap(),
            [true]
        );
        let failure =
            run_filter_with("        return #dam.history(task) > 0", &task, &fixed).unwrap_err();
        let FilterFault::Raised { message } = failure.fault else {
            panic!("{:?}", failure.fault);
        };
        assert!(message.contains("dam.history"), "{message}");
    }
}
