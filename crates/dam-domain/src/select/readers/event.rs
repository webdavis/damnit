use crate::select::Context;
use crate::select::values::{Item, Values};
use crate::{Event, Object};

fn of_event(o: &Object, read: impl FnOnce(&Event) -> Values) -> Values {
    o.as_event().map_or(Values::Lacks, read)
}

pub(crate) fn start(o: &Object, c: &dyn Context) -> Values {
    let zone = c.now().time_zone();
    of_event(o, |e| Values::of(e.start.instant(zone).map(Item::Point)))
}

pub(crate) fn end(o: &Object, c: &dyn Context) -> Values {
    let zone = c.now().time_zone();
    of_event(o, |e| Values::of(e.end.instant(zone).map(Item::Point)))
}

pub(crate) fn location(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| {
        Values::texts(e.location.iter().filter(|l| !l.is_empty()))
    })
}

pub(crate) fn status(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| Values::texts([e.status.as_str()]))
}

pub(crate) fn transparency(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| Values::texts([e.transparency.as_str()]))
}

pub(crate) fn visibility(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| Values::texts([e.visibility.as_str()]))
}

pub(crate) fn event_type(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| Values::texts([e.event_type.as_str()]))
}

pub(crate) fn response(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| {
        Values::texts(
            e.attendees
                .iter()
                .filter(|a| a.is_self)
                .map(|a| a.response.as_str()),
        )
    })
}

pub(crate) fn organizer_email(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| Values::texts(e.organizer.iter().map(|p| &p.email)))
}

pub(crate) fn attendee_emails(o: &Object, _: &dyn Context) -> Values {
    of_event(o, |e| Values::texts(e.attendees.iter().map(|a| &a.email)))
}
