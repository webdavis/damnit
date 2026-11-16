use dam_domain::{Item, ItemHash, Timestamp};
use serde::Serialize;

use super::waiting::{ItemRow, age};
use crate::view::{Line, display};

#[derive(Serialize)]
pub(super) struct SetRow<'a> {
    #[serde(serialize_with = "display")]
    set: &'a ItemHash,
    from: &'a str,
    count: usize,
    #[serde(serialize_with = "display")]
    created: &'a Timestamp,
    age: i64,
    items: Vec<ItemRow<'a>>,
}

impl<'a> SetRow<'a> {
    pub(super) fn new(
        set: &'a ItemHash,
        members: &[&'a Item],
        items: Vec<ItemRow<'a>>,
        now: Timestamp,
    ) -> Option<SetRow<'a>> {
        let first = members.iter().min_by_key(|item| item.created)?;
        Some(SetRow {
            set,
            from: &first.author.name,
            count: members.len(),
            created: &first.created,
            age: now.duration_since(first.created).as_secs().max(0),
            items,
        })
    }

    pub(super) fn line(&self) -> Line {
        let short = self.set.short();
        Line::Row(format!(
            "  {short}  {} changes  {}  {}  dam show {short}",
            self.count,
            self.from,
            age(self.age)
        ))
    }
}
