use dam_application::ConfigError;

use super::insert::{Indent, insert};
use super::lines::{indent_of_line, starts_its_line};
use super::locate::{Located, TableLayout, Unwritable, locate};
use super::lua_text::table_key;
use super::remove::remove;

pub(super) struct Section<'a> {
    pub(super) file: &'a str,
    pub(super) table: &'a str,
    pub(super) entry: &'a str,
    pub(super) by_hand: String,
}

impl Section<'_> {
    pub(super) fn add(
        &self,
        text: &str,
        entry: &dyn Fn(&Indent) -> String,
    ) -> Result<String, ConfigError> {
        let located = self.locate(text)?;
        let per_level = indent_per_level(text, &located);
        match &located.section {
            Some(table) if self.position(table).is_some() => Err(self.held_already()),
            Some(table) => Ok(insert(text, table, &per_level, entry)),
            None => Ok(insert(text, &located.root, &per_level, |indent| {
                let inner = indent.deeper();
                let table = table_key(self.table);
                let entry = entry(&inner);
                format!(
                    "{table} = {{\n{}{entry},\n{}}}",
                    inner.leading, indent.leading
                )
            })),
        }
    }

    pub(super) fn remove(&self, text: &str, held: bool) -> Result<String, ConfigError> {
        let located = self.locate(text)?;
        let written = located
            .section
            .as_ref()
            .and_then(|table| Some((table, self.position(table)?)));
        let (file, table, entry) = (self.file, self.table, self.entry);
        match written {
            Some((layout, index)) => Ok(remove(text, layout, index)),
            None if held => Err(ConfigError::Invalid(format!(
                "{file} holds {table} {entry} under a key dam cannot edit. {}",
                self.by_hand
            ))),
            None => Err(ConfigError::Invalid(format!(
                "{file} holds no {table} {entry}"
            ))),
        }
    }

    pub(super) fn held_already(&self) -> ConfigError {
        ConfigError::AlreadyHeld(format!(
            "{} already holds {} {}",
            self.file, self.table, self.entry
        ))
    }

    fn position(&self, table: &TableLayout) -> Option<usize> {
        table
            .entries
            .iter()
            .position(|e| e.key.as_deref() == Some(self.entry))
    }

    fn locate(&self, text: &str) -> Result<Located, ConfigError> {
        locate(text, self.table).map_err(|why| {
            let place = match why {
                Unwritable::NotLua => format!("{} has Lua that dam cannot edit", self.file),
                Unwritable::RootBuiltInCode => {
                    format!("{} returns a table built in code", self.file)
                }
                Unwritable::SectionBuiltInCode => {
                    format!("the {} table in {} is built in code", self.table, self.file)
                }
            };
            ConfigError::Invalid(format!(
                "{place}, and dam edits only a table written out in full. {}",
                self.by_hand
            ))
        })
    }
}

fn indent_per_level(text: &str, located: &Located) -> String {
    match located.root.entries.first() {
        Some(e) if starts_its_line(text, e.span.start) => {
            let first = indent_of_line(text, e.span.start);
            if first.is_empty() { "  " } else { first }.to_string()
        }
        _ => "  ".to_string(),
    }
}
