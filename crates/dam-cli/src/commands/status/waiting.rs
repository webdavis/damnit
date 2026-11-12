use dam_domain::{Item, ItemHash, Oid, Timestamp};
use serde::Serialize;

use super::rows::ChangeRow;
use crate::view::display;

#[derive(Serialize)]
pub(super) struct ItemRow<'a> {
    #[serde(serialize_with = "display")]
    hash: &'a ItemHash,
    item: &'static str,
    #[serde(skip)]
    target: &'a Oid,
    #[serde(flatten)]
    change: ChangeRow<'a>,
    #[serde(serialize_with = "display")]
    created: &'a Timestamp,
    age: i64,
}

impl<'a> ItemRow<'a> {
    pub(super) fn new(item: &'a Item, now: Timestamp, full: bool) -> ItemRow<'a> {
        ItemRow {
            hash: &item.hash,
            item: item.kind().as_str(),
            target: item.target(),
            change: ChangeRow::new(&item.change, Some(&item.author), full),
            created: &item.created,
            age: now.duration_since(item.created).as_secs().max(0),
        }
    }

    pub(super) fn line(&self, owner: &str) -> String {
        let target = self.target;
        let id = if target == self.hash.oid() {
            self.hash.short().to_string()
        } else {
            format!("{}  {}", self.hash.short(), target.short())
        };
        format!(
            "{}  {}",
            self.change.status_line_naming(&id, owner),
            age(self.age)
        )
    }
}

fn age(seconds: i64) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    match seconds {
        s if s < MINUTE => format!("{s}s"),
        s if s < HOUR => format!("{}m", s / MINUTE),
        s if s < DAY => format!("{}h", s / HOUR),
        s => format!("{}d", s / DAY),
    }
}

#[cfg(test)]
mod tests;
