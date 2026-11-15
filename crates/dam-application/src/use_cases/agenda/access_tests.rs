use dam_domain::Permission::{EventRead, TaskRead};
use dam_domain::{Event, Object, Path, When};
use jiff::tz::TimeZone;

use super::Window;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, oid};

fn meeting(n: u8, subject: &str, path: &str) -> Object {
    let at = |t: &str| When::At(t.parse::<jiff::Zoned>().unwrap());
    let mut e = Event::new(
        oid(n),
        subject,
        at("2026-09-25T14:00:00+00:00[UTC]"),
        at("2026-09-25T15:00:00+00:00[UTC]"),
    );
    e.base.path = Path::parse(path).unwrap();
    Object::Event(e)
}

#[test]
fn a_token_sees_no_meeting_outside_its_grants() {
    let s = Scenario::default();
    s.working(meeting(1, "design review", "work"));
    s.working(meeting(2, "dentist", "home"));
    let token = holder(vec![allowed(&[TaskRead, EventRead], Some(&["work"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let window = Window {
        from: "2026-09-25T00:00:00Z".parse().unwrap(),
        to: "2026-09-26T00:00:00Z".parse().unwrap(),
    };
    let seen: Vec<String> = dam
        .agenda(None, window, &TimeZone::UTC)
        .unwrap()
        .into_iter()
        .map(|s| s.event.base.subject)
        .collect();
    assert_eq!(seen, ["design review"]);
}
