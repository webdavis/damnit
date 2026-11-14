use std::fmt::Display;

use dam_adapters::to_wire;
use dam_domain::{Object, When};
use serde::{Serialize, Serializer};

pub(crate) fn display<T: Display, S: Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

pub(crate) fn optional_display<T: Display, S: Serializer>(
    value: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value
        .as_ref()
        .map(ToString::to_string)
        .serialize(serializer)
}

pub(crate) fn displays<T: Display, S: Serializer>(
    values: &&[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(values.iter().map(ToString::to_string))
}

pub(crate) fn document(object: &Object) -> impl Serialize + use<> {
    let mut document = to_wire(object, None);
    if let (Some(task), Some(t)) = (document.task.as_mut(), object.as_task()) {
        task.due = t.due.as_ref().map(When::to_rfc3339);
    }
    if let (Some(event), Some(e)) = (document.event.as_mut(), object.as_event()) {
        event.start = e.start.to_rfc3339();
        event.end = e.end.to_rfc3339();
    }
    document
}

pub(crate) fn wire<S: Serializer>(object: &&Object, serializer: S) -> Result<S::Ok, S::Error> {
    document(object).serialize(serializer)
}

pub(crate) fn optional_wire<S: Serializer>(
    object: &Option<&Object>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    object.map(document).serialize(serializer)
}

pub(super) fn wires<S: Serializer>(objects: &&[Object], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(objects.iter().map(document))
}
