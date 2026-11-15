use dam_domain::{
    Change, Field, Object, changed_fields, kind_change, merge_fields, touched_fields,
};

use crate::errors::UseCaseError;
use crate::ports::ObjectRepository;
use crate::use_cases::remove::plan_for;

type Split = (Vec<Field>, Vec<Field>);

pub(super) fn set_back(
    objects: &dyn ObjectRepository,
    change: &Change,
) -> Result<Split, UseCaseError> {
    let working = objects.get(change.oid())?;
    match change {
        Change::Update { before, after } => update(objects, working, before, after),
        Change::Create { after } => create(objects, working, after),
        Change::Delete { before } => delete(objects, working, before),
    }
}

fn update(
    objects: &dyn ObjectRepository,
    working: Option<Object>,
    before: &Object,
    after: &Object,
) -> Result<Split, UseCaseError> {
    let fields = changed_fields(before, after);
    let Some(working) = working.filter(|w| kind_change(w, after).is_none()) else {
        return Ok((Vec::new(), fields));
    };
    if kind_change(before, after).is_some() {
        return whole_object(objects, &working, before, after, fields);
    }
    let (back, kept): Split = fields.into_iter().partition(|f| f.same(&working, after));
    if !back.is_empty() {
        objects.put(&merge_fields(&working, before, &back))?;
    }
    Ok((back, kept))
}

fn whole_object(
    objects: &dyn ObjectRepository,
    working: &Object,
    before: &Object,
    after: &Object,
    fields: Vec<Field>,
) -> Result<Split, UseCaseError> {
    if working != after {
        return Ok((Vec::new(), fields));
    }
    objects.put(before)?;
    Ok((fields, Vec::new()))
}

fn create(
    objects: &dyn ObjectRepository,
    working: Option<Object>,
    after: &Object,
) -> Result<Split, UseCaseError> {
    let fields = fields_it_holds(after);
    let untouched = working.as_ref() == Some(after) && nothing_points_at(objects, after)?;
    if !untouched {
        return Ok((Vec::new(), fields));
    }
    objects.delete(after.oid())?;
    Ok((fields, Vec::new()))
}

fn delete(
    objects: &dyn ObjectRepository,
    working: Option<Object>,
    before: &Object,
) -> Result<Split, UseCaseError> {
    let fields = fields_it_holds(before);
    if working.is_some() {
        return Ok((Vec::new(), fields));
    }
    objects.put(before)?;
    Ok((fields, Vec::new()))
}

fn fields_it_holds(object: &Object) -> Vec<Field> {
    touched_fields(&Change::Create {
        after: object.clone(),
    })
}

fn nothing_points_at(
    objects: &dyn ObjectRepository,
    object: &Object,
) -> Result<bool, UseCaseError> {
    let plan = plan_for(objects, object)?;
    Ok(plan.moved_up.is_empty() && plan.dependents.is_empty() && plan.attached.is_empty())
}
