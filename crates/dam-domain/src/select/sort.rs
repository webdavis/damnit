use std::cmp::Ordering;

use super::Context;
use super::closest::closest_field;
use super::error::{FilterError, Problem};
use super::field::{Holds, SelectField};
use super::fields::select_field;
use super::values::{Item, Values};
use crate::Object;

#[derive(Clone, Debug, Default)]
pub struct Sort {
    keys: Vec<SortKey>,
}

#[derive(Clone, Debug)]
struct SortKey {
    field: &'static SelectField,
    descending: bool,
}

type Read = Vec<Option<Vec<Item>>>;

impl Sort {
    pub fn read(texts: &[String]) -> Result<Sort, FilterError> {
        let keys = texts
            .iter()
            .flat_map(|text| text.split(','))
            .map(key)
            .collect::<Result<_, _>>()?;
        Ok(Sort { keys })
    }

    pub fn arrange(&self, objects: Vec<Object>, context: &dyn Context) -> Vec<Object> {
        if self.keys.is_empty() {
            return objects;
        }
        let mut read: Vec<(Read, Object)> = objects
            .into_iter()
            .map(|o| (self.read_keys(&o, context), o))
            .collect();
        read.sort_by(|(a, _), (b, _)| self.compare(a, b));
        read.into_iter().map(|(_, o)| o).collect()
    }

    fn read_keys(&self, object: &Object, context: &dyn Context) -> Read {
        self.keys
            .iter()
            .map(|k| sortable((k.field.read)(object, context), k.field.holds))
            .collect()
    }

    fn compare(&self, a: &Read, b: &Read) -> Ordering {
        self.keys
            .iter()
            .zip(a.iter().zip(b))
            .map(|(key, (a, b))| match (a, b) {
                (Some(a), Some(b)) if key.descending => b.cmp(a),
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            })
            .find(|o| o.is_ne())
            .unwrap_or(Ordering::Equal)
    }
}

fn sortable(values: Values, holds: Holds) -> Option<Vec<Item>> {
    let Values::Has(items) = values else {
        return None;
    };
    let mut items: Vec<Item> = items.into_iter().map(Item::folded).collect();
    if holds == Holds::Many {
        items.sort();
    }
    Some(items).filter(|items| !items.is_empty())
}

fn key(text: &str) -> Result<SortKey, FilterError> {
    let (name, direction) = text.split_once(':').unwrap_or((text, ""));
    let error = |problem| FilterError {
        field: name.into(),
        value: text.into(),
        problem,
    };
    let field = select_field(name).ok_or_else(|| {
        error(Problem::UnknownField {
            closest: closest_field(name),
        })
    })?;
    match direction {
        "" => Ok(SortKey {
            field,
            descending: false,
        }),
        "desc" => Ok(SortKey {
            field,
            descending: true,
        }),
        _ => Err(error(Problem::BadValue)),
    }
}
