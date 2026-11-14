use dam_application::EditFields;
use dam_domain::{Date, Object, Oid};
use dam_toml::{StrictTable, parse_toml};

use super::reader::template_reader;
use kind_fields::{event_fields, task_fields};

pub(crate) fn parse_template(
    text: &str,
    current: &Object,
    today: Date,
    tz: &jiff::tz::TimeZone,
) -> Result<EditFields, String> {
    let table = parse_toml(text)?;
    let reader = template_reader(text, &table, current)?;

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
        if value != b.recurrence.as_ref().map(ToString::to_string) {
            fields.recurrence = Some(value);
        }
    }
    Ok(fields)
}

fn labels(reader: &StrictTable, current: &Object, fields: &mut EditFields) -> Result<(), String> {
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

fn depends(reader: &StrictTable, current: &Object, fields: &mut EditFields) -> Result<(), String> {
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

mod kind_fields;
