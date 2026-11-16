use dam_application::Dam;
use dam_domain::{Change, Field, touched_fields};
use serde_json::{Map, Value};

use crate::view::{category_document, document, field_value};

const ON_EVERY_CHANGE: [Field; 9] = [
    Field::Subject,
    Field::Path,
    Field::Labels,
    Field::Done,
    Field::Priority,
    Field::Due,
    Field::Start,
    Field::End,
    Field::Kind,
];

pub(super) fn after_values(dam: &Dam, change: &Change) -> Map<String, Value> {
    let mut values = Map::new();
    let Some(after) = change.after() else {
        return values;
    };
    let doc = serde_json::to_value(document(after)).unwrap_or_default();
    for field in touched_fields(change) {
        match field {
            Field::Path => {
                values.insert("parent".into(), field_value(&doc, "parent").clone());
            }
            Field::Labels => {
                let category = dam.worked_out_from_object(after).category;
                values.insert("category".into(), category_document(&category));
            }
            _ if ON_EVERY_CHANGE.contains(&field) => {}
            _ => {
                values.insert(
                    field.as_str().into(),
                    field_value(&doc, field.as_str()).clone(),
                );
            }
        }
    }
    values
}
