use std::fmt::Display;

use mlua::{Table, Value};

pub(super) struct StrictMap {
    file: String,
    place: Option<String>,
    entries: Vec<(String, Value)>,
}

impl StrictMap {
    pub(super) fn root(file: &str, table: &Table) -> Result<StrictMap, String> {
        let entries = map_entries(table)
            .ok_or_else(|| format!("{file} must return a table of named settings"))?;
        Ok(StrictMap {
            file: file.to_string(),
            place: None,
            entries,
        })
    }

    pub(super) fn only(&self, known: &[&str]) -> Result<(), String> {
        match self.entries().find(|(key, _)| !known.contains(key)) {
            Some((key, _)) => Err(self.refuse(format_args!("unknown key {key}"))),
            None => Ok(()),
        }
    }

    pub(super) fn entries(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub(super) fn get(&self, key: &str) -> Option<&Value> {
        self.entries().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    pub(super) fn refuse(&self, what: impl Display) -> String {
        match &self.place {
            None => format!("{what} at the top level of {}", self.file),
            Some(place) => format!("{what} in {place} of {}", self.file),
        }
    }

    pub(super) fn string(&self, key: &str) -> Result<Option<String>, String> {
        self.get(key)
            .map(|v| text(v).ok_or_else(|| self.refuse(format_args!("{key} must be a string"))))
            .transpose()
    }

    pub(super) fn bool(&self, key: &str) -> Result<Option<bool>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::Boolean(b)) => Ok(Some(*b)),
            Some(_) => Err(self.refuse(format_args!("{key} must be true or false"))),
        }
    }

    pub(super) fn strings(&self, key: &str) -> Result<Option<Vec<String>>, String> {
        self.get(key)
            .map(|v| {
                list(v)
                    .and_then(|items| items.iter().map(text).collect())
                    .ok_or_else(|| self.refuse(format_args!("{key} must be a list of strings")))
            })
            .transpose()
    }

    pub(super) fn map(&self, key: &str) -> Result<Option<StrictMap>, String> {
        self.get(key).map(|v| self.child(key, v)).transpose()
    }

    pub(super) fn maps_in(&self, key: &str) -> Result<Vec<(String, StrictMap)>, String> {
        let Some(parent) = self.map(key)? else {
            return Ok(Vec::new());
        };
        parent
            .entries()
            .map(|(name, v)| Ok((name.to_string(), parent.child(name, v)?)))
            .collect()
    }

    fn child(&self, key: &str, value: &Value) -> Result<StrictMap, String> {
        match value {
            Value::Table(table) => map_entries(table)
                .map(|entries| self.at(self.place_of(key), entries))
                .ok_or_else(|| self.refuse(format_args!("{key} must be a table of named entries"))),
            _ => Err(self.refuse(format_args!("{key} must be a table"))),
        }
    }

    fn at(&self, place: String, entries: Vec<(String, Value)>) -> StrictMap {
        StrictMap {
            file: self.file.clone(),
            place: Some(place),
            entries,
        }
    }

    fn place_of(&self, key: &str) -> String {
        match &self.place {
            None => key.to_string(),
            Some(place) => format!("{place}.{key}"),
        }
    }
}

pub(super) fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
        _ => None,
    }
}

pub(super) fn list(value: &Value) -> Option<Vec<Value>> {
    let Value::Table(t) = value else { return None };
    let items = t
        .sequence_values::<Value>()
        .collect::<mlua::Result<Vec<_>>>()
        .ok()?;
    (t.pairs::<Value, Value>().count() == items.len()).then_some(items)
}

fn map_entries(table: &Table) -> Option<Vec<(String, Value)>> {
    let mut entries = table
        .pairs::<Value, Value>()
        .map(|pair| {
            let (key, value) = pair.ok()?;
            Some((text(&key)?, value))
        })
        .collect::<Option<Vec<_>>>()?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Some(entries)
}
