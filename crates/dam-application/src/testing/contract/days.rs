use dam_domain::{Day, TaskState};
use jiff::civil::date;

use crate::ports::DayRepository;
use crate::testing::oid;

pub fn day_repository_contract(store: &dyn DayRepository) {
    let on = date(2026, 11, 11);
    assert_eq!(store.day(on).unwrap(), None);
    let mut day = Day::open(on);
    day.plan = vec![oid(1)];
    day.proposals = vec![oid(2)];
    day.decisions = vec!["skip the gym".into()];
    day.tasks = vec![TaskState {
        oid: oid(1),
        stalls: 2,
        carried_forward: true,
    }];
    store.put_day(&day).unwrap();
    store.put_day(&Day::open(date(2026, 11, 12))).unwrap();
    assert_eq!(store.day(on).unwrap(), Some(day.clone()));
    day.closed_at = Some("2026-11-11T22:00:00Z".parse().unwrap());
    store.put_day(&day).unwrap();
    assert_eq!(store.day(on).unwrap(), Some(day));
    assert_eq!(store.days().unwrap().len(), 2);
}
