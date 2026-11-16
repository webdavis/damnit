use dam_domain::{Date, Object, When, date_to_set};
use dam_toml::StrictTable;
use jiff::Zoned;
use toml::Table;

use crate::commands::parsing::date_forms;

const KEYS_A_TASK_TEMPLATE_MAY_CARRY: [&str; 9] = [
    "subject",
    "body",
    "labels",
    "depends",
    "recurrence",
    "priority",
    "due",
    "deadline",
    "attached",
];
const KEYS_AN_EVENT_TEMPLATE_MAY_CARRY: [&str; 8] = [
    "subject",
    "body",
    "labels",
    "depends",
    "recurrence",
    "start",
    "end",
    "location",
];

pub(super) fn template_reader<'a>(
    text: &'a str,
    table: &'a Table,
    current: &Object,
) -> Result<StrictTable<'a>, String> {
    let known: &[&str] = match current {
        Object::Task(_) => &KEYS_A_TASK_TEMPLATE_MAY_CARRY,
        Object::Event(_) => &KEYS_AN_EVENT_TEMPLATE_MAY_CARRY,
    };
    StrictTable::new(text, table, known)
}

pub(super) fn when(
    reader: &StrictTable,
    key: &str,
    text: &str,
    today: Date,
    now: &Zoned,
) -> Result<When, String> {
    date_to_set(text, today, now).ok_or_else(|| {
        reader.refuse(
            key,
            format_args!("{key} cannot be {text:?}; {}", date_forms()),
        )
    })
}
