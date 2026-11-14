use dam_domain::Object;
use jiff::tz::TimeZone;
use serde::Serialize;

use super::serialize::{wire, wires};
use super::{Human, View};
use crate::output::object_cells;

#[derive(Serialize)]
pub(crate) struct ObjectList<'a> {
    #[serde(serialize_with = "wires")]
    pub(crate) objects: &'a [Object],
    #[serde(skip)]
    pub(crate) authors: &'a [&'a str],
    #[serde(skip)]
    pub(crate) zone: &'a TimeZone,
}

impl View for ObjectList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.objects
                .iter()
                .zip(self.authors)
                .map(|(o, author)| {
                    let mut cells = object_cells(o, self.zone);
                    cells.insert(4, author.to_string());
                    cells
                })
                .collect(),
            "no objects match",
        )
    }
}

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct ObjectRow<'a>(
    #[serde(serialize_with = "wire")] pub(crate) &'a Object,
    #[serde(skip)] pub(crate) &'a TimeZone,
);

impl View for ObjectRow<'_> {
    fn human(&self) -> Human {
        Human::Table(vec![object_cells(self.0, self.1)])
    }
}
