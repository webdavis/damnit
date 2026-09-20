use dam_application::EditFields;
use dam_domain::{Date, Object, Oid, Priority};

use super::reader::TemplateReader;

/// Every key a saved template may carry, per kind. A key outside the set is
/// refused, because the operator who wrote it meant it to do something.
const TASK_KEYS: [&str; 9] = [
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
const EVENT_KEYS: [&str; 8] = [
    "subject",
    "body",
    "labels",
    "depends",
    "recurrence",
    "start",
    "end",
    "location",
];

/// Reads a saved template back as the difference from `current`: a field left
/// unchanged in the text stays out of the result, and an optional field emptied
/// in the text clears it. The error names the line that is wrong.
pub fn parse_template(
    text: &str,
    current: &Object,
    today: Date,
    tz: &jiff::tz::TimeZone,
) -> Result<EditFields, String> {
    let reader = TemplateReader::parse(text)?;
    let known: &[&str] = match current {
        Object::Task(_) => &TASK_KEYS,
        Object::Event(_) => &EVENT_KEYS,
    };
    if let Some(key) = reader.unknown_key(known) {
        return Err(reader.refuse(key, format_args!("unknown field {key}")));
    }

    let mut fields = EditFields::default();
    let b = current.base();
    if let Some(s) = reader.string("subject")?
        && s != b.subject
    {
        fields.subject = Some(s);
    }
    if let Some(s) = reader.string("body")?
        && s != b.body
    {
        fields.body = Some(s);
    }
    match current {
        Object::Task(t) => task_fields(&reader, today, tz, t, &mut fields)?,
        Object::Event(e) => event_fields(&reader, today, tz, e, &mut fields)?,
    }
    labels(&reader, current, &mut fields)?;
    depends(&reader, current, &mut fields)?;
    if let Some(s) = reader.string("recurrence")? {
        let value = (!s.is_empty()).then_some(s);
        if value != b.recurrence {
            fields.recurrence = Some(value);
        }
    }
    Ok(fields)
}

fn labels(
    reader: &TemplateReader,
    current: &Object,
    fields: &mut EditFields,
) -> Result<(), String> {
    let Some(labels) = reader.string_array("labels", "an array of strings")? else {
        return Ok(());
    };
    let held = &current.base().labels;
    fields.add_labels = labels
        .iter()
        .filter(|l| !held.contains(*l))
        .cloned()
        .collect();
    fields.remove_labels = held
        .iter()
        .filter(|l| !labels.contains(l))
        .cloned()
        .collect();
    Ok(())
}

fn depends(
    reader: &TemplateReader,
    current: &Object,
    fields: &mut EditFields,
) -> Result<(), String> {
    let Some(texts) = reader.string_array("depends", "an array of oids")? else {
        return Ok(());
    };
    let oids = texts
        .iter()
        .map(|t| {
            Oid::parse(t).map_err(|e| reader.refuse("depends", format_args!("depends: {e:?}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let held = &current.base().depends;
    fields.add_depends = oids.iter().filter(|o| !held.contains(o)).cloned().collect();
    fields.remove_depends = held.iter().filter(|o| !oids.contains(o)).cloned().collect();
    Ok(())
}

fn task_fields(
    reader: &TemplateReader,
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
            false => Some(reader.when("due", &s, today, tz)?),
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
    if let Some(s) = reader.string("attach")? {
        let value = match s.is_empty() {
            true => None,
            false => Some(
                Oid::parse(&s)
                    .map_err(|e| reader.refuse("attach", format_args!("attach: {e:?}")))?,
            ),
        };
        if value != t.event {
            fields.attach = Some(value);
        }
    }
    Ok(())
}

fn event_fields(
    reader: &TemplateReader,
    today: Date,
    tz: &jiff::tz::TimeZone,
    e: &dam_domain::Event,
    fields: &mut EditFields,
) -> Result<(), String> {
    if let Some(s) = reader.string("start")? {
        let value = reader.when("start", &s, today, tz)?;
        if value != e.start {
            fields.start = Some(value);
        }
    }
    if let Some(s) = reader.string("end")? {
        let value = reader.when("end", &s, today, tz)?;
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
