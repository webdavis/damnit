use super::Context;
use super::date::DateTest;
use super::values::Item;
use crate::{Oid, Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Want {
    Empty,
    Any,
    Contains(String),
    Is(String),
    Range(Option<i64>, Option<i64>),
    Under(Path),
    Id(Oid),
    ListedBy(String),
    Yes(bool),
    Date(DateTest),
    Group(String, Option<String>),
}

impl Want {
    pub(crate) fn holds(&self, item: &Item, context: &dyn Context) -> bool {
        match (self, item) {
            (Want::Any, _) => true,
            (Want::Contains(needle), Item::Text(t)) => t.to_lowercase().contains(needle),
            (Want::Is(v), Item::Text(t)) => v == t,
            (Want::Range(lo, hi), Item::Number(n)) => {
                lo.is_none_or(|lo| *n >= lo) && hi.is_none_or(|hi| *n <= hi)
            }
            (Want::Under(folder), Item::Path(p)) => p.is_within(folder),
            (Want::Id(want), Item::Id(id)) => want == id,
            (Want::ListedBy(filter), Item::Id(id)) => context.lists(filter, id),
            (Want::Yes(want), Item::Yes(yes)) => want == yes,
            (Want::Date(date), Item::Point(at)) => date.holds(*at, context),
            (Want::Group(_, Some(label)), Item::Text(t)) => label == t,
            (Want::Group(group, None), Item::Text(t)) => context
                .categories()
                .category_of_name(group)
                .is_some_and(|c| c.values.contains(t)),
            _ => false,
        }
    }
}
