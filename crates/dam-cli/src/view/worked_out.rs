use std::collections::BTreeMap;

use dam_domain::WorkedOut;
use serde_json::{Map, Value, json};

pub(crate) fn stamp_worked_out(row: &mut Value, worked: &WorkedOut) {
    let Some(row) = row.as_object_mut() else {
        return;
    };
    row.insert("category".into(), category(&worked.category));
    for (key, yes) in [("overdue", worked.overdue), ("blocked", worked.blocked)] {
        if let Some(yes) = yes {
            row.insert(key.into(), yes.into());
        }
    }
}

pub(crate) fn category(groups: &BTreeMap<String, Vec<String>>) -> Value {
    let groups: Map<String, Value> = groups
        .iter()
        .map(|(group, labels)| match labels.as_slice() {
            [label] => (group.clone(), json!(label)),
            several => (group.clone(), json!(several)),
        })
        .collect();
    Value::Object(groups)
}
