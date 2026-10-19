use super::Change;
use crate::{Field, Kind, Object};

pub fn changed_fields(before: &Object, after: &Object) -> Vec<Field> {
    Field::ALL
        .into_iter()
        .filter(|field| !field.same(before, after))
        .collect()
}

pub fn merge_fields(ours: &Object, theirs: &Object, fields: &[Field]) -> Object {
    if kind_change(ours, theirs).is_some() {
        return ours.clone();
    }
    let mut out = ours.clone();
    for field in fields {
        field.copy(theirs, &mut out);
    }
    out
}

pub fn kind_change(ours: &Object, theirs: &Object) -> Option<(Kind, Kind)> {
    let (o, t) = (ours.kind(), theirs.kind());
    (o != t).then_some((o, t))
}

pub fn touched_fields(change: &Change) -> Vec<Field> {
    match change {
        Change::Update { before, after } => changed_fields(before, after),
        Change::Create { after } => fields_a_create_sets(after),
        Change::Delete { .. } => Vec::new(),
    }
}

fn fields_a_create_sets(object: &Object) -> Vec<Field> {
    let always_set: Vec<Field> = match object {
        Object::Task(_) => vec![Field::Subject],
        Object::Event(_) => vec![Field::Subject, Field::Start, Field::End],
    };
    let defaults = with_every_field_at_default(object);
    let non_default = changed_fields(&defaults, object)
        .into_iter()
        .filter(|field| !always_set.contains(field));
    always_set.iter().copied().chain(non_default).collect()
}

fn with_every_field_at_default(object: &Object) -> Object {
    let oid = object.base().oid.clone();
    match object {
        Object::Task(_) => Object::Task(crate::Task::new(oid, "")),
        Object::Event(e) => {
            Object::Event(crate::Event::new(oid, "", e.start.clone(), e.end.clone()))
        }
    }
}
