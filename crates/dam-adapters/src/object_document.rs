use dam_domain::{Object, When};
use dam_protocol::WireObject;

use crate::wire::to_wire;

pub fn object_document(object: &Object) -> WireObject {
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
