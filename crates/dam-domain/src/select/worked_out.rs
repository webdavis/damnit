use std::collections::BTreeMap;

use super::Context;
use super::fields::select_field;
use super::values::{Item, Values};
use crate::{Date, Object};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkedOut {
    pub category: BTreeMap<String, Vec<String>>,
    pub overdue: Option<bool>,
    pub blocked: Option<bool>,
    pub days_until_due: Option<i32>,
    pub days_until_deadline: Option<i32>,
    pub children_open: Option<usize>,
}

pub fn worked_out(object: &Object, context: &dyn Context) -> WorkedOut {
    WorkedOut {
        category: category(object, context),
        overdue: yes_or_no("overdue", object, context),
        blocked: yes_or_no("blocked", object, context),
        days_until_due: object
            .as_task()
            .and_then(|t| days_until(t.due.as_ref()?.day_in(context.now().time_zone()), context)),
        days_until_deadline: object
            .as_task()
            .and_then(|t| days_until(t.deadline?, context)),
        children_open: Some(context.open_children(object.oid())),
    }
}

fn days_until(day: Date, context: &dyn Context) -> Option<i32> {
    context
        .now()
        .date()
        .until(day)
        .ok()
        .map(|span| span.get_days())
}

fn read(key: &str, object: &Object, context: &dyn Context) -> Vec<Item> {
    let field = select_field(key).expect("a worked-out field is in the field table");
    match (field.read)(object, context) {
        Values::Has(items) => items,
        Values::Lacks | Values::Always => Vec::new(),
    }
}

fn yes_or_no(key: &str, object: &Object, context: &dyn Context) -> Option<bool> {
    read(key, object, context)
        .into_iter()
        .find_map(|item| match item {
            Item::Yes(yes) => Some(yes),
            _ => None,
        })
}

fn category(object: &Object, context: &dyn Context) -> BTreeMap<String, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in read("category", object, context) {
        let Item::Text(label) = item else { continue };
        if let Some(group) = context.categories().category_of(&label) {
            groups.entry(group.name.clone()).or_default().push(label);
        }
    }
    groups
}
