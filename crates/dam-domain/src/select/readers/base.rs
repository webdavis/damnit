use crate::Object;
use crate::select::Context;
use crate::select::values::{Item, Values};

pub(crate) fn kind(o: &Object, _: &dyn Context) -> Values {
    Values::texts([o.kind().as_str()])
}

pub(crate) fn subject(o: &Object, _: &dyn Context) -> Values {
    Values::texts([&o.base().subject])
}

pub(crate) fn body(o: &Object, _: &dyn Context) -> Values {
    Values::texts(Some(&o.base().body).filter(|b| !b.is_empty()))
}

pub(crate) fn path(o: &Object, _: &dyn Context) -> Values {
    Values::of([Item::Path(o.base().path.clone())])
}

pub(crate) fn labels(o: &Object, _: &dyn Context) -> Values {
    Values::texts(&o.base().labels)
}

pub(crate) fn parent(o: &Object, _: &dyn Context) -> Values {
    Values::of(o.base().parent.iter().cloned().map(Item::Id))
}

pub(crate) fn depends(o: &Object, _: &dyn Context) -> Values {
    Values::of(o.base().depends.iter().cloned().map(Item::Id))
}

pub(crate) fn reminders(o: &Object, _: &dyn Context) -> Values {
    Values::of(
        o.base()
            .reminders
            .iter()
            .map(|r| Item::Number(r.minutes_before)),
    )
}

pub(crate) fn own_id(o: &Object, _: &dyn Context) -> Values {
    Values::of([Item::Id(o.oid().clone())])
}

pub(crate) fn author(o: &Object, c: &dyn Context) -> Values {
    Values::texts([c.author(o.oid())])
}
