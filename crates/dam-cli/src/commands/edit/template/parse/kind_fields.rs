use dam_application::EditFields;
use dam_domain::{Date, Oid, Priority};
use dam_toml::StrictTable;

use super::super::reader::when;

pub(super) fn task_fields(
    reader: &StrictTable,
    today: Date,
    tz: &jiff::tz::TimeZone,
    t: &dam_domain::Task,
    fields: &mut EditFields,
) -> Result<(), String> {
    if let Some(v) = reader.get("priority") {
        let n = v
            .as_integer()
            .and_then(|n| u8::try_from(n).ok())
            .and_then(|n| Priority::new(n).ok())
            .ok_or_else(|| reader.refuse("priority", "priority must be 1, 2, 3 or 4"))?;
        if n != t.priority {
            fields.priority = Some(n);
        }
    }
    if let Some(s) = reader.string("due")? {
        let value = match s.is_empty() {
            true => None,
            false => Some(when(reader, "due", &s, today, tz)?),
        };
        if value != t.due {
            fields.due = Some(value);
        }
    }
    if let Some(s) = reader.string("deadline")? {
        let value = match s.is_empty() {
            true => None,
            false => Some(
                s.parse::<Date>()
                    .map_err(|e| reader.refuse("deadline", format_args!("deadline: {e}")))?,
            ),
        };
        if value != t.deadline {
            fields.deadline = Some(value);
        }
    }
    if let Some(s) = reader.string("attached")? {
        let value = match s.is_empty() {
            true => None,
            false => Some(
                Oid::parse(&s)
                    .map_err(|e| reader.refuse("attached", format_args!("attached: {e:?}")))?,
            ),
        };
        if value != t.event {
            fields.attach = Some(value);
        }
    }
    Ok(())
}

pub(super) fn event_fields(
    reader: &StrictTable,
    today: Date,
    tz: &jiff::tz::TimeZone,
    e: &dam_domain::Event,
    fields: &mut EditFields,
) -> Result<(), String> {
    if let Some(s) = reader.string("start")? {
        let value = when(reader, "start", &s, today, tz)?;
        if value != e.start {
            fields.start = Some(value);
        }
    }
    if let Some(s) = reader.string("end")? {
        let value = when(reader, "end", &s, today, tz)?;
        if value != e.end {
            fields.end = Some(value);
        }
    }
    if let Some(s) = reader.string("location")? {
        let value = (!s.is_empty()).then_some(s);
        if value != e.location {
            fields.location = Some(value);
        }
    }
    Ok(())
}
