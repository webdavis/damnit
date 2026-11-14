use dam_application::{ByHand, ConfigError, EditedTable};

use super::insert::{Indent, insert};
use super::lines::{indent_of_line, starts_its_line};
use super::locate::{Located, TableLayout, locate};
use super::lua_text::table_key;
use super::remove::remove;

pub(super) struct Section<'a> {
    pub(super) table: EditedTable,
    pub(super) entry: &'a str,
    pub(super) by_hand: ByHand,
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
                let table = table_key(lua_key(self.table));
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
        match written {
            Some((layout, index)) => Ok(remove(text, layout, index)),
            None if held => Err(ConfigError::HeldUnderUneditableKey {
                table: self.table,
                entry: self.entry.to_string(),
                by_hand: self.by_hand.clone(),
            }),
            None => Err(ConfigError::NotHeld {
                table: self.table,
                entry: self.entry.to_string(),
            }),
        }
    }

    pub(super) fn held_already(&self) -> ConfigError {
        ConfigError::AlreadyHeld {
            table: self.table,
            entry: self.entry.to_string(),
        }
    }

    fn position(&self, table: &TableLayout) -> Option<usize> {
        table
            .entries
            .iter()
            .position(|e| e.key.as_deref() == Some(self.entry))
    }

    fn locate(&self, text: &str) -> Result<Located, ConfigError> {
        locate(text, lua_key(self.table)).map_err(|why| ConfigError::CannotEdit {
            table: self.table,
            why,
            by_hand: self.by_hand.clone(),
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

fn lua_key(table: EditedTable) -> &'static str {
    match table {
        EditedTable::Remotes => "remote",
        EditedTable::Tokens => "token",
    }
}
