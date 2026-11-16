use std::cmp::Ordering;

use dam_domain::{Date, Object, Timestamp};

fn date_of(o: &Object) -> Option<Date> {
    match o {
        Object::Task(t) => t.due.as_ref().map(|d| d.date()),
        Object::Event(e) => Some(e.start.date()),
    }
}

fn focused_at(o: &Object, today: Date) -> Option<Timestamp> {
    o.as_task()?.focus_on(today).map(|f| f.set_at)
}

pub(super) fn order(a: &Object, b: &Object, today: Date) -> Ordering {
    match (focused_at(a, today), focused_at(b, today)) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
    .then_with(|| a.base().path.cmp(&b.base().path))
    .then_with(|| match (date_of(a), date_of(b)) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    })
    .then_with(|| {
        let pa = a.as_task().map(|t| t.priority.get()).unwrap_or(u8::MAX);
        let pb = b.as_task().map(|t| t.priority.get()).unwrap_or(u8::MAX);
        pa.cmp(&pb)
    })
    .then_with(|| a.base().subject.cmp(&b.base().subject))
}
