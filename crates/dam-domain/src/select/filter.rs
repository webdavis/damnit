use std::collections::BTreeMap;

use super::Context;
use super::condition::Condition;
use super::error::{FilterError, Problem};
use super::field::Holds;
use super::table::{FilterTable, TableValue};
use super::want::SavedUse;
use crate::{Categories, Object};

#[derive(Clone, Debug, Default)]
pub struct Filter {
    conditions: Vec<Condition>,
    any: Vec<Filter>,
}

impl Filter {
    pub fn read(table: &FilterTable) -> Result<Filter, FilterError> {
        let mut conditions = Vec::new();
        for (key, value) in &table.fields {
            let texts = value.texts();
            if texts.is_empty() {
                return Err(Condition::read(key, "").err().unwrap_or_else(|| empty(key)));
            }
            let read: Vec<Condition> = texts
                .iter()
                .map(|text| Condition::read(key, text))
                .collect::<Result<_, _>>()?;
            refuse_two_at_once(&read)?;
            conditions.extend(read);
        }
        let any = table
            .any
            .iter()
            .map(Filter::read)
            .collect::<Result<_, _>>()?;
        Ok(Filter { conditions, any })
    }

    pub fn write(&self) -> FilterTable {
        let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for c in &self.conditions {
            fields
                .entry(c.key().into())
                .or_default()
                .push(c.text().into());
        }
        let fields = fields
            .into_iter()
            .map(|(key, mut texts)| match texts.len() {
                1 => (key, TableValue::One(texts.remove(0))),
                _ => (key, TableValue::Many(texts)),
            })
            .collect();
        FilterTable {
            fields,
            any: self.any.iter().map(Filter::write).collect(),
        }
    }

    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    pub fn any(&self) -> &[Filter] {
        &self.any
    }

    pub fn saved_filters(&self) -> Vec<&str> {
        self.named(SavedUse::Any)
    }

    pub fn selected_filters(&self) -> Vec<&str> {
        self.named(SavedUse::Selected)
    }

    pub fn listed_filters(&self) -> Vec<&str> {
        self.named(SavedUse::Listed)
    }

    pub fn event_filters(&self) -> Vec<&str> {
        self.named(SavedUse::Events)
    }

    fn named(&self, use_: SavedUse) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        let own = self.conditions.iter().flat_map(|c| c.saved_filters(use_));
        let nested = self.any.iter().flat_map(|f| f.named(use_));
        for name in own.chain(nested) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    pub fn check(&self, categories: &Categories) -> Result<(), FilterError> {
        self.conditions
            .iter()
            .try_for_each(|c| c.check(categories))?;
        self.any.iter().try_for_each(|f| f.check(categories))
    }

    pub fn matches(&self, object: &Object, context: &dyn Context) -> bool {
        self.conditions.iter().all(|c| c.holds(object, context))
            && (self.any.is_empty() || self.any.iter().any(|f| f.matches(object, context)))
    }
}

fn refuse_two_at_once(read: &[Condition]) -> Result<(), FilterError> {
    let mut values: Vec<&Condition> = read
        .iter()
        .filter(|c| c.field.holds != Holds::Many && c.is_one_value())
        .collect();
    values.dedup_by_key(|c| c.text());
    let [first, second, ..] = values.as_slice() else {
        return Ok(());
    };
    let fix = first.field.takes_a_list().then(|| {
        let texts: Vec<&str> = values.iter().map(|c| c.text()).collect();
        texts.join(",")
    });
    Err(second.error(Problem::TwoAtOnce { fix }))
}

fn empty(key: &str) -> FilterError {
    FilterError {
        field: key.into(),
        value: String::new(),
        problem: Problem::BadValue,
    }
}
