use std::fmt::Display;

use toml::{Table, Value};

pub(crate) enum Place {
    TopLevel,
    Section(String),
}

pub(crate) struct StrictTable<'a> {
    table: &'a Table,
    place: Place,
}

pub(crate) fn parse_toml(text: &str) -> Result<Table, String> {
    text.parse().map_err(|e: toml::de::Error| match e.span() {
        Some(span) => {
            let (line, column) = one_based_line_and_column_of_byte(text, span.start);
            format!("line {line}, column {column}: {}", e.message())
        }
        None => e.message().to_string(),
    })
}

impl<'a> StrictTable<'a> {
    pub(crate) fn new(
        table: &'a Table,
        place: Place,
        known: &[&str],
    ) -> Result<StrictTable<'a>, String> {
        let strict = StrictTable { table, place };
        strict.only(known)?;
        Ok(strict)
    }

    pub(crate) fn only(&self, known: &[&str]) -> Result<(), String> {
        match self.first_unknown_key(known) {
            Some(key) => Err(self.refuse(format_args!("unknown key {key}"))),
            None => Ok(()),
        }
    }

    pub(crate) fn first_unknown_key(&self, known: &[&str]) -> Option<&'a str> {
        self.table
            .keys()
            .map(String::as_str)
            .find(|key| !known.contains(key))
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = (&'a str, &'a Value)> {
        self.table.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub(crate) fn refuse(&self, what: impl Display) -> String {
        match &self.place {
            Place::TopLevel => format!("{what} at the top level"),
            Place::Section(name) => format!("{what} in [{name}]"),
        }
    }

    pub(crate) fn section(
        &self,
        key: &str,
        known: &[&str],
    ) -> Result<Option<StrictTable<'a>>, String> {
        let Some(section) = self.table_at(key, Place::Section(key.to_string()))? else {
            return Ok(None);
        };
        section.only(known)?;
        Ok(Some(section))
    }

    pub(crate) fn tables_in(&self, key: &str) -> Result<Vec<(&'a str, StrictTable<'a>)>, String> {
        let Some(parent) = self.table_at(key, Place::Section(key.to_string()))? else {
            return Ok(Vec::new());
        };
        parent
            .table
            .iter()
            .map(|(name, v)| match v {
                Value::Table(table) => Ok((
                    name.as_str(),
                    StrictTable {
                        table,
                        place: Place::Section(format!("{key}.{name}")),
                    },
                )),
                _ => Err(parent.refuse(format_args!("{name} must be a table"))),
            })
            .collect()
    }

    fn table_at(&self, key: &str, place: Place) -> Result<Option<StrictTable<'a>>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::Table(table)) => Ok(Some(StrictTable { table, place })),
            Some(_) => Err(self.refuse(format_args!("{key} must be a table"))),
        }
    }

    pub(crate) fn bool(&self, key: &str) -> Result<Option<bool>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::Boolean(b)) => Ok(Some(*b)),
            Some(_) => Err(self.refuse(format_args!("{key} must be true or false"))),
        }
    }

    pub(crate) fn string(&self, key: &str) -> Result<Option<String>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(self.refuse(format_args!("{key} must be a string"))),
        }
    }

    pub(crate) fn string_array(
        &self,
        key: &str,
        what: &str,
    ) -> Result<Option<Vec<String>>, String> {
        let Some(value) = self.table.get(key) else {
            return Ok(None);
        };
        let items = value
            .as_array()
            .and_then(|a| a.iter().map(|x| x.as_str().map(str::to_string)).collect());
        items
            .map(Some)
            .ok_or_else(|| self.refuse(format_args!("{key} must be {what}")))
    }
}

fn one_based_line_and_column_of_byte(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..clamped_to_a_char_boundary(text, offset)];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    (line, column)
}

fn clamped_to_a_char_boundary(text: &str, offset: usize) -> usize {
    let mut end = offset.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}
