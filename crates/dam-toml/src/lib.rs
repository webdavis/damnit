use std::fmt::Display;

use toml::{Table, Value};

pub enum Place {
    Line,
    TopLevel,
    Section(String),
}

pub struct StrictTable<'a> {
    text: &'a str,
    table: &'a Table,
    place: Place,
}

pub fn parse_toml(text: &str) -> Result<Table, String> {
    text.parse().map_err(|e: toml::de::Error| match e.span() {
        Some(span) => {
            let (line, column) = one_based_line_and_column_of_byte(text, span.start);
            format!("line {line}, column {column}: {}", e.message())
        }
        None => e.message().to_string(),
    })
}

impl<'a> StrictTable<'a> {
    pub fn new(
        text: &'a str,
        table: &'a Table,
        place: Place,
        known: &[&str],
    ) -> Result<StrictTable<'a>, String> {
        let strict = StrictTable { text, table, place };
        strict.only(known)?;
        Ok(strict)
    }

    pub fn only(&self, known: &[&str]) -> Result<(), String> {
        match self.first_unknown_key(known) {
            Some(key) => Err(self.refuse(key, format_args!("unknown key {key}"))),
            None => Ok(()),
        }
    }

    pub fn first_unknown_key(&self, known: &[&str]) -> Option<&'a str> {
        self.table
            .keys()
            .map(String::as_str)
            .find(|key| !known.contains(key))
    }

    pub fn keys(&self) -> impl Iterator<Item = (&'a str, &'a Value)> {
        self.table.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn get(&self, key: &str) -> Option<&'a Value> {
        self.table.get(key)
    }

    pub fn refuse(&self, key: &str, what: impl Display) -> String {
        match &self.place {
            Place::Line => match one_based_line_of_bare_key(self.text, key) {
                Some(line) => format!("line {line}: {what}"),
                None => what.to_string(),
            },
            Place::TopLevel => format!("{what} at the top level"),
            Place::Section(name) => format!("{what} in [{name}]"),
        }
    }

    pub fn section(&self, key: &str, known: &[&str]) -> Result<Option<StrictTable<'a>>, String> {
        let Some(section) = self.table_at(key, Place::Section(key.to_string()))? else {
            return Ok(None);
        };
        section.only(known)?;
        Ok(Some(section))
    }

    pub fn tables_in(&self, key: &str) -> Result<Vec<(&'a str, StrictTable<'a>)>, String> {
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
                        text: self.text,
                        table,
                        place: Place::Section(format!("{key}.{name}")),
                    },
                )),
                _ => Err(parent.refuse(name, format_args!("{name} must be a table"))),
            })
            .collect()
    }

    fn table_at(&self, key: &str, place: Place) -> Result<Option<StrictTable<'a>>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::Table(table)) => Ok(Some(StrictTable {
                text: self.text,
                table,
                place,
            })),
            Some(_) => Err(self.refuse(key, format_args!("{key} must be a table"))),
        }
    }

    pub fn bool(&self, key: &str) -> Result<Option<bool>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::Boolean(b)) => Ok(Some(*b)),
            Some(_) => Err(self.refuse(key, format_args!("{key} must be true or false"))),
        }
    }

    pub fn string(&self, key: &str) -> Result<Option<String>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(self.refuse(key, format_args!("{key} must be a string"))),
        }
    }

    pub fn string_array(&self, key: &str, what: &str) -> Result<Option<Vec<String>>, String> {
        let Some(value) = self.table.get(key) else {
            return Ok(None);
        };
        let items = value
            .as_array()
            .and_then(|a| a.iter().map(|x| x.as_str().map(str::to_string)).collect());
        items
            .map(Some)
            .ok_or_else(|| self.refuse(key, format_args!("{key} must be {what}")))
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

fn one_based_line_of_bare_key(text: &str, key: &str) -> Option<usize> {
    text.lines()
        .position(|line| {
            let line = line.trim_start();
            line.strip_prefix(key)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
        .map(|i| i + 1)
}

#[cfg(test)]
mod tests;
