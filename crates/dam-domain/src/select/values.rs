use crate::{Oid, Path, Timestamp};

pub(crate) enum Values {
    Lacks,
    Always,
    Has(Vec<Item>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Item {
    Text(String),
    Number(i64),
    Point(Timestamp),
    Id(Oid),
    Path(Path),
    Yes(bool),
}

impl Item {
    pub(crate) fn folded(self) -> Item {
        match self {
            Item::Text(text) => Item::Text(text.to_lowercase()),
            item => item,
        }
    }
}

impl Values {
    pub(crate) fn of(items: impl IntoIterator<Item = Item>) -> Values {
        Values::Has(items.into_iter().collect())
    }

    pub(crate) fn texts<S: ToString>(texts: impl IntoIterator<Item = S>) -> Values {
        Values::of(texts.into_iter().map(|t| Item::Text(t.to_string())))
    }
}
