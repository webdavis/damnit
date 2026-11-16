use super::Context;
use super::closest::closest_field;
use super::error::{FilterError, Problem};
use super::field::{FieldType, SelectField};
use super::fields::select_field;
use super::read_value::wants;
use super::values::Values;
use super::want::{SavedUse, Want};
use crate::{Categories, Object};

#[derive(Clone, Debug)]
pub struct Condition {
    pub(crate) field: &'static SelectField,
    pub(crate) text: String,
    negated: bool,
    wants: Vec<Want>,
}

impl Condition {
    pub fn read(key: &str, text: &str) -> Result<Condition, FilterError> {
        let field = select_field(key).ok_or_else(|| FilterError {
            field: key.into(),
            value: text.into(),
            problem: Problem::UnknownField {
                closest: closest_field(key),
            },
        })?;
        let (negated, rest) = match text.strip_prefix("not:") {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let negatable = !matches!(field.ty, FieldType::YesNo);
        let wants = wants(field, rest)
            .filter(|_| negatable || !negated)
            .filter(|w| !negated || !w.contains(&Want::Any))
            .ok_or_else(|| field_error(field, text, Problem::BadValue))?;
        Ok(Condition {
            field,
            text: text.into(),
            negated,
            wants,
        })
    }

    pub fn key(&self) -> &'static str {
        self.field.key
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn holds(&self, object: &Object, context: &dyn Context) -> bool {
        if self.wants.contains(&Want::Any) {
            return true;
        }
        match (self.field.read)(object, context) {
            Values::Always => true,
            Values::Lacks => self.negated && !self.wants.contains(&Want::Empty),
            Values::Has(items) => {
                let hit = self.wants.iter().any(|want| match want {
                    Want::Empty => items.is_empty(),
                    want => items.iter().any(|item| want.holds(item, context)),
                });
                hit != self.negated
            }
        }
    }

    pub fn check(&self, categories: &Categories) -> Result<(), FilterError> {
        for want in &self.wants {
            let Want::Group(group, label) = want else {
                continue;
            };
            let Some(category) = categories.category_of_name(group) else {
                let groups = categories.all().iter().map(|c| c.name.clone()).collect();
                return Err(self.error(Problem::NotAGroup { groups }));
            };
            if label.as_ref().is_some_and(|l| !category.values.contains(l)) {
                let (group, labels) = (group.clone(), category.values.clone());
                return Err(self.error(Problem::NotInGroup { group, labels }));
            }
        }
        Ok(())
    }

    pub(crate) fn saved_filters(&self, use_: SavedUse) -> impl Iterator<Item = &str> {
        self.wants.iter().filter_map(move |w| w.saved_filter(use_))
    }

    pub(crate) fn is_one_value(&self) -> bool {
        !self.negated && matches!(self.wants.as_slice(), [want] if want.is_one_value())
    }

    pub(crate) fn admits_word(&self, word: &str) -> bool {
        let hit = self.wants.iter().any(|want| match want {
            Want::Any => true,
            Want::Is(value) => value == word,
            Want::Yes(yes) => yes.to_string() == word,
            _ => false,
        });
        hit != self.negated
    }

    pub(crate) fn error(&self, problem: Problem) -> FilterError {
        field_error(self.field, &self.text, problem)
    }
}

fn field_error(field: &SelectField, value: &str, problem: Problem) -> FilterError {
    FilterError {
        field: field.key.into(),
        value: value.into(),
        problem,
    }
}
