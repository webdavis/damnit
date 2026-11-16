use super::Context;
use super::date::DateTest;
use super::values::Item;
use crate::{Oid, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SavedUse {
    Any,
    Selected,
    Listed,
    Events,
}

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
    Selects(String),
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
            (Want::Selects(filter), Item::Id(id)) => context.selects(filter, id),
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

    pub(crate) fn saved_filter(&self, use_: SavedUse) -> Option<&str> {
        match (self, use_) {
            (Want::Selects(name), SavedUse::Selected | SavedUse::Any)
            | (Want::ListedBy(name), SavedUse::Listed | SavedUse::Any) => Some(name),
            (Want::Date(date), SavedUse::Events | SavedUse::Any) => date.saved_filter(),
            _ => None,
        }
    }

    pub(crate) fn is_one_value(&self) -> bool {
        match self {
            Want::Is(_) | Want::Under(_) | Want::Id(_) | Want::Yes(_) => true,
            Want::Range(lo, hi) => lo.is_some() && lo == hi,
            Want::Date(date) => date.is_one_day_or_instant(),
            _ => false,
        }
    }
}
