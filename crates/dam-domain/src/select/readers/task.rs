use crate::select::Context;
use crate::select::values::{Item, Values};
use crate::{Date, Object, Task};

fn of_task(o: &Object, read: impl FnOnce(&Task) -> Values) -> Values {
    o.as_task().map_or(Values::Lacks, read)
}

fn day(day: Option<Date>, context: &dyn Context) -> Values {
    let zone = context.now().time_zone();
    Values::of(
        day.and_then(|d| d.to_zoned(zone.clone()).ok())
            .map(|z| Item::Point(z.timestamp())),
    )
}

pub(crate) fn done(o: &Object, _: &dyn Context) -> Values {
    o.as_task().map_or(Values::Always, |t| {
        Values::of([Item::Yes(t.completion.is_done())])
    })
}

pub(crate) fn overdue(o: &Object, c: &dyn Context) -> Values {
    of_task(o, |t| {
        let zone = c.now().time_zone();
        let today = c.now().date().to_zoned(zone.clone()).ok();
        let late = match (t.due.as_ref().and_then(|d| d.instant(zone)), today) {
            (Some(due), Some(today)) => due < today.timestamp(),
            _ => false,
        };
        Values::of([Item::Yes(!t.completion.is_done() && late)])
    })
}

pub(crate) fn blocked(o: &Object, c: &dyn Context) -> Values {
    of_task(o, |t| {
        Values::of([Item::Yes(t.base.depends.iter().any(|d| c.holds_up(d)))])
    })
}

pub(crate) fn priority(o: &Object, _: &dyn Context) -> Values {
    of_task(o, |t| Values::of([Item::Number(t.priority.get().into())]))
}

pub(crate) fn due(o: &Object, c: &dyn Context) -> Values {
    let zone = c.now().time_zone();
    of_task(o, |t| {
        Values::of(
            t.due
                .as_ref()
                .and_then(|d| d.instant(zone))
                .map(Item::Point),
        )
    })
}

pub(crate) fn deadline(o: &Object, c: &dyn Context) -> Values {
    of_task(o, |t| day(t.deadline, c))
}

pub(crate) fn focus_day(o: &Object, c: &dyn Context) -> Values {
    of_task(o, |t| day(t.focus.as_ref().map(|f| f.day), c))
}

pub(crate) fn completed_at(o: &Object, _: &dyn Context) -> Values {
    of_task(o, |t| Values::of(t.completion.at().map(Item::Point)))
}

pub(crate) fn estimate(o: &Object, _: &dyn Context) -> Values {
    of_task(o, |t| {
        Values::of(t.estimate.map(|m| Item::Number(m.into())))
    })
}

pub(crate) fn attached(o: &Object, _: &dyn Context) -> Values {
    of_task(o, |t| Values::of(t.event.clone().map(Item::Id)))
}
