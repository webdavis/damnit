use dam_domain::Permission::TaskRead;
use dam_domain::{Object, Reminder, Timestamp, When};
use jiff::tz::TimeZone;

use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, task_in};
use crate::use_cases::agenda::Window;

#[test]
fn a_token_hears_no_reminder_of_an_object_outside_its_grants() {
    let s = Scenario::default();
    for (n, subject, path) in [(1, "report", "work"), (2, "laundry", "home")] {
        let mut t = task_in(n, subject, path);
        t.due = Some(When::Day("2026-01-02".parse().unwrap()));
        t.base.reminders = vec![Reminder { minutes_before: 0 }];
        s.working(Object::Task(t));
    }
    let token = holder(vec![allowed(&[TaskRead], Some(&["work"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let window = Window {
        from: Timestamp::UNIX_EPOCH,
        to: "2027-01-01T00:00:00Z".parse().unwrap(),
    };
    let heard: Vec<String> = dam
        .due_reminders(window, &TimeZone::UTC)
        .unwrap()
        .into_iter()
        .map(|r| r.subject)
        .collect();
    assert_eq!(heard, ["report"]);
}
