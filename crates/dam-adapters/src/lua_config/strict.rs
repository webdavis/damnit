use dam_application::{ConfigError, Expected, Misread, Place};
use mlua::{Table, Value};

use super::shape::{list, map_entries, text};

pub(super) struct StrictMap {
    file: String,
    place: Option<String>,
    entries: Vec<(String, Value)>,
}

impl StrictMap {
    pub(super) fn root(file: &str, table: &Table) -> Result<StrictMap, ConfigError> {
        let entries = map_entries(table).ok_or_else(|| ConfigError::NotNamedSettings {
            file: file.to_string(),
        })?;
        Ok(StrictMap {
            file: file.to_string(),
            place: None,
            entries,
        })
    }

    pub(super) fn only(&self, known: &[&str]) -> Result<(), Misread> {
        match self.entries().find(|(key, _)| !known.contains(key)) {
            Some((key, _)) => Err(Misread::UnknownKey {
                key: key.to_string(),
                at: self.place(),
            }),
            None => Ok(()),
        }
    }

    pub(super) fn entries(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub(super) fn get(&self, key: &str) -> Option<&Value> {
        self.entries().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    pub(super) fn place(&self) -> Place {
        Place {
            file: self.file.clone(),
            table: self.place.clone(),
        }
    }

    pub(super) fn wrong(&self, key: &str, expected: Expected) -> Misread {
        Misread::WrongType {
            key: key.to_string(),
            at: self.place(),
            expected,
        }
    }

    pub(super) fn missing(&self, key: &str) -> Misread {
        Misread::Missing {
            key: key.to_string(),
            at: self.place(),
        }
    }

    pub(super) fn string(&self, key: &str) -> Result<Option<String>, Misread> {
        self.get(key)
            .map(|v| text(v).ok_or_else(|| self.wrong(key, Expected::Text)))
            .transpose()
    }

    pub(super) fn bool(&self, key: &str) -> Result<Option<bool>, Misread> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::Boolean(b)) => Ok(Some(*b)),
            Some(_) => Err(self.wrong(key, Expected::TrueOrFalse)),
        }
    }

    pub(super) fn strings(&self, key: &str) -> Result<Option<Vec<String>>, Misread> {
        self.get(key)
            .map(|v| {
                list(v)
                    .and_then(|items| items.iter().map(text).collect())
                    .ok_or_else(|| self.wrong(key, Expected::ListOfTexts))
            })
            .transpose()
    }

    pub(super) fn map(&self, key: &str) -> Result<Option<StrictMap>, Misread> {
        self.get(key).map(|v| self.child(key, v)).transpose()
    }

    pub(super) fn maps_in(&self, key: &str) -> Result<Vec<(String, StrictMap)>, Misread> {
        let Some(parent) = self.map(key)? else {
            return Ok(Vec::new());
        };
        parent
            .entries()
            .map(|(name, v)| Ok((name.to_string(), parent.child(name, v)?)))
            .collect()
    }

    pub(super) fn maps_listed(&self, key: &str) -> Result<Option<Vec<StrictMap>>, Misread> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        let wrong = || self.wrong(key, Expected::ListOfTables);
        let items = list(value).ok_or_else(wrong)?;
        let place = self.place_of(key);
        items
            .iter()
            .map(|item| match item {
                Value::Table(table) => {
                    map_entries(table).map(|entries| self.at(place.clone(), entries))
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(Some)
            .ok_or_else(wrong)
    }

    pub(super) fn child(&self, key: &str, value: &Value) -> Result<StrictMap, Misread> {
        match value {
            Value::Table(table) => map_entries(table)
                .map(|entries| self.at(self.place_of(key), entries))
                .ok_or_else(|| self.wrong(key, Expected::TableOfNamedEntries)),
            _ => Err(self.wrong(key, Expected::Table)),
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
