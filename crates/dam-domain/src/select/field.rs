use super::Context;
use super::values::Values;
use crate::Object;

#[derive(Clone, Copy, Debug)]
pub enum FieldType {
    Text,
    Choice(fn() -> Vec<&'static str>),
    Number(i64, i64),
    Length,
    TextSet,
    Path,
    Reference,
    YesNo,
    Date,
    Category,
    FilterName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Has {
    Task,
    Event,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holds {
    One,
    OneOrNone,
    Many,
}

#[derive(Debug)]
pub struct SelectField {
    pub key: &'static str,
    pub ty: FieldType,
    pub has: Has,
    pub holds: Holds,
    pub(crate) read: fn(&Object, &dyn Context) -> Values,
}

impl SelectField {
    pub fn shapes(&self) -> Vec<String> {
        let mut shapes: Vec<String> = match self.ty {
            FieldType::Choice(choices) => choices().into_iter().map(String::from).collect(),
            ty => ty.shapes().iter().map(|s| s.to_string()).collect(),
        };
        if self.holds != Holds::One {
            shapes.push("none".into());
        }
        shapes
    }

    pub(crate) fn date_shapes(&self) -> Vec<&'static str> {
        match self.ty {
            FieldType::Date => FieldType::Date.shapes().to_vec(),
            _ => Vec::new(),
        }
    }

    pub fn takes_a_list(&self) -> bool {
        !matches!(
            self.ty,
            FieldType::Text | FieldType::Date | FieldType::YesNo
        )
    }
}

impl FieldType {
    fn shapes(self) -> &'static [&'static str] {
        match self {
            FieldType::Text => &["TEXT", "not:TEXT"],
            FieldType::Choice(_) => &[],
            FieldType::Number(..) => &["N", "N,M", "N..M", "..M", "N..", "not:N"],
            FieldType::Length => &["30m", "2h", "30m..2h", "..5m", "1h..", "not:30m"],
            FieldType::TextSet => &["VALUE", "A,B", "not:VALUE", "=VALUE"],
            FieldType::Path => &["FOLDER/", "A/,B/", "not:FOLDER/"],
            FieldType::Reference => &["ID", "ID,ID", "in:FILTER", "not:ID"],
            FieldType::YesNo => &["true", "false", "any"],
            FieldType::Date => &[
                "2026-11-20",
                "2026-11-20T09:00",
                "2026-W47",
                "2026-11",
                "2026",
                "today",
                "today+3",
                "week",
                "week+1",
                "month-1",
                "year",
                "now+2h",
                "A..B",
                "..B",
                "A..",
                "on:thu",
                "on:1tue",
                "on:-1fri",
                "on:15",
                "on:-1",
                "on:FILTER",
                "not:DATE",
            ],
            FieldType::Category => &["GROUP", "GROUP=LABEL", "GROUP=A,B", "not:GROUP"],
            FieldType::FilterName => &["NAME", "A,B", "not:NAME"],
        }
    }
}
