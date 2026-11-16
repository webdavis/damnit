use crate::{Date, Event, Object, Oid, Task, When};

pub fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

pub fn task(b: u8) -> Task {
    Task::new(oid(b), format!("task {b}"))
}

pub fn due(mut t: Task, day: Date) -> Task {
    t.due = Some(When::Day(day));
    t
}

pub fn event(b: u8, start: &str) -> Event {
    let at = |s: &str| When::At(s.parse::<jiff::Zoned>().unwrap());
    Event::new(oid(b), format!("event {b}"), at(start), at(start))
}

pub fn labelled(mut o: Object, labels: &[&str]) -> Object {
    o.base_mut().labels = labels.iter().map(|l| l.to_string()).collect();
    o
}
