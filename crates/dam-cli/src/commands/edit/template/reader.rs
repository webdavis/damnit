use dam_domain::{Date, Object, When};
use toml::Table;

use dam_adapters::{Place, StrictTable};

const KEYS_A_TASK_TEMPLATE_MAY_CARRY: [&str; 9] = [
    "subject",
    "body",
    "labels",
    "depends",
    "recurrence",
    "priority",
    "due",
    "deadline",
    "attach",
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
    StrictTable::new(text, table, Place::Line, known)
}

pub(super) fn when(
    reader: &StrictTable,
    key: &str,
    text: &str,
    today: Date,
    tz: &jiff::tz::TimeZone,
) -> Result<When, String> {
    When::parse_human(text, today, tz).map_err(|e| reader.refuse(key, format_args!("{key}: {e}")))
}
