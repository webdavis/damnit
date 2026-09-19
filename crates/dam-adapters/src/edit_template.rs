use dam_application::EditFields;
use dam_domain::{Date, Object, Oid, Priority, When};
use toml::{Table, Value};

/// Renders `object` as a TOML template for `edit -e`: a comment header naming the
/// oid and kind, then one line per editable field. Empty string means none.
pub fn render_template(object: &Object) -> String {
    let b = object.base();
    let kind = match object {
        Object::Task(_) => "task",
        Object::Event(_) => "event",
    };
    let mut out = format!(
        "# dam edit {}  ({kind})\n# Comment lines are ignored. Save to apply, or quit without saving to cancel.\n",
        b.oid.short()
    );
    let mut line = |key: &str, value: Value| out.push_str(&format!("{key} = {value}\n"));
    let opt = |s: Option<String>| Value::String(s.unwrap_or_default());
    line("subject", Value::String(b.subject.clone()));
    line("body", Value::String(b.body.clone()));
    match object {
        Object::Task(t) => {
            line("priority", Value::Integer(i64::from(t.priority.get())));
            line("due", opt(t.due.as_ref().map(When::to_text)));
            line("deadline", opt(t.deadline.map(|d| d.to_string())));
        }
        Object::Event(e) => {
            line("start", Value::String(e.start.to_text()));
            line("end", Value::String(e.end.to_text()));
            line("location", opt(e.location.clone()));
        }
    }
    line(
        "labels",
        Value::Array(b.labels.iter().map(|l| Value::String(l.clone())).collect()),
    );
    line(
        "depends",
        Value::Array(
            b.depends
                .iter()
                .map(|d| Value::String(d.to_string()))
                .collect(),
        ),
    );
    line("recurrence", opt(b.recurrence.clone()));
    if let Object::Task(t) = object {
        line("attach", opt(t.event.as_ref().map(ToString::to_string)));
    }
    out
}

/// Reads a saved template back as the difference from `current`: a field left
/// unchanged in the text stays out of the result, and an optional field emptied
/// in the text clears it. The error names the line that is wrong.
pub fn parse_template(
    text: &str,
    current: &Object,
    today: Date,
    tz: &jiff::tz::TimeZone,
) -> Result<EditFields, String> {
    let table: Table = text.parse().map_err(|e: toml::de::Error| {
        let line = e.span().map(|s| line_of(text, s.start)).unwrap_or(0);
        format!("line {line}: {}", e.message())
    })?;
    let at = |key: &str| line_of_key(text, key);
    let string = |key: &str| -> Result<Option<String>, String> {
        match table.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(format!("line {}: {key} must be a string", at(key))),
        }
    };
    let when = |key: &str, s: &str| {
        When::parse_human(s, today, tz).map_err(|e| format!("line {}: {key}: {e}", at(key)))
    };

    let mut fields = EditFields::default();
    let b = current.base();
    if let Some(s) = string("subject")?
        && s != b.subject
    {
        fields.subject = Some(s);
    }
    if let Some(s) = string("body")?
        && s != b.body
    {
        fields.body = Some(s);
    }
    match current {
        Object::Task(t) => parse_task_fields(&table, &at, &string, &when, t, &mut fields)?,
        Object::Event(e) => parse_event_fields(&string, &when, e, &mut fields)?,
    }
    if let Some(v) = table.get("labels") {
        let labels = string_array(v)
            .ok_or_else(|| format!("line {}: labels must be an array of strings", at("labels")))?;
        fields.add_labels = labels
            .iter()
            .filter(|l| !b.labels.contains(*l))
            .cloned()
            .collect();
        fields.remove_labels = b
            .labels
            .iter()
            .filter(|l| !labels.contains(l))
            .cloned()
            .collect();
    }
    if let Some(v) = table.get("depends") {
        let texts = string_array(v)
            .ok_or_else(|| format!("line {}: depends must be an array of oids", at("depends")))?;
        let oids = texts
            .iter()
            .map(|t| Oid::parse(t).map_err(|e| format!("line {}: depends: {e:?}", at("depends"))))
            .collect::<Result<Vec<_>, _>>()?;
        fields.add_depends = oids
            .iter()
            .filter(|o| !b.depends.contains(o))
            .cloned()
            .collect();
        fields.remove_depends = b
            .depends
            .iter()
            .filter(|o| !oids.contains(o))
            .cloned()
            .collect();
    }
    if let Some(s) = string("recurrence")? {
        let value = if s.is_empty() { None } else { Some(s) };
        if value != b.recurrence {
            fields.recurrence = Some(value);
        }
    }
    Ok(fields)
}

type StringField<'a> = dyn Fn(&str) -> Result<Option<String>, String> + 'a;
type WhenField<'a> = dyn Fn(&str, &str) -> Result<When, String> + 'a;

fn parse_task_fields(
    table: &Table,
    at: &dyn Fn(&str) -> usize,
    string: &StringField,
    when: &WhenField,
    t: &dam_domain::Task,
    fields: &mut EditFields,
) -> Result<(), String> {
    if let Some(v) = table.get("priority") {
        let n = v
            .as_integer()
            .and_then(|n| u8::try_from(n).ok())
            .and_then(|n| Priority::new(n).ok())
            .ok_or_else(|| format!("line {}: priority must be 1, 2, 3 or 4", at("priority")))?;
        if n != t.priority {
            fields.priority = Some(n);
        }
    }
    if let Some(s) = string("due")? {
        let value = if s.is_empty() {
            None
        } else {
            Some(when("due", &s)?)
        };
        if value != t.due {
            fields.due = Some(value);
        }
    }
    if let Some(s) = string("deadline")? {
        let value = if s.is_empty() {
            None
        } else {
            Some(
                s.parse::<Date>()
                    .map_err(|e| format!("line {}: deadline: {e}", at("deadline")))?,
            )
        };
        if value != t.deadline {
            fields.deadline = Some(value);
        }
    }
    if let Some(s) = string("attach")? {
        let value = if s.is_empty() {
            None
        } else {
            Some(Oid::parse(&s).map_err(|e| format!("line {}: attach: {e:?}", at("attach")))?)
        };
        if value != t.event {
            fields.attach = Some(value);
        }
    }
    Ok(())
}

fn parse_event_fields(
    string: &StringField,
    when: &WhenField,
    e: &dam_domain::Event,
    fields: &mut EditFields,
) -> Result<(), String> {
    if let Some(s) = string("start")? {
        let value = when("start", &s)?;
        if value != e.start {
            fields.start = Some(value);
        }
    }
    if let Some(s) = string("end")? {
        let value = when("end", &s)?;
        if value != e.end {
            fields.end = Some(value);
        }
    }
    if let Some(s) = string("location")? {
        let value = if s.is_empty() { None } else { Some(s) };
        if value != e.location {
            fields.location = Some(value);
        }
    }
    Ok(())
}

fn string_array(v: &Value) -> Option<Vec<String>> {
    v.as_array()?
        .iter()
        .map(|x| x.as_str().map(str::to_string))
        .collect()
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

fn line_of_key(text: &str, key: &str) -> usize {
    text.lines()
        .position(|l| {
            let l = l.trim_start();
            l.strip_prefix(key)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
        .map(|i| i + 1)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use dam_application::EditFields;
    use dam_domain::{Object, Oid, Priority, Task, When};
    use jiff::civil::date;

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    fn task() -> Object {
        let mut t = Task::new(oid(1), "milk");
        t.priority = Priority::new(2).unwrap();
        t.base.labels.insert("errand".into());
        Object::Task(t)
    }

    fn tz() -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }

    #[test]
    fn an_unchanged_template_parses_to_no_edits() {
        let text = super::render_template(&task());
        let fields = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap();
        assert_eq!(fields, EditFields::default());
    }

    #[test]
    fn changed_values_become_edits_and_labels_diff() {
        let text = super::render_template(&task())
            .replace("subject = \"milk\"", "subject = \"oat milk\"")
            .replace("due = \"\"", "due = \"tomorrow\"")
            .replace("labels = [\"errand\"]", "labels = [\"grocery\"]");
        let fields = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap();
        assert_eq!(fields.subject.as_deref(), Some("oat milk"));
        assert_eq!(fields.due, Some(Some(When::Day(date(2026, 9, 19)))));
        assert_eq!(fields.add_labels, vec!["grocery".to_string()]);
        assert_eq!(fields.remove_labels, vec!["errand".to_string()]);
    }

    #[test]
    fn a_bad_line_is_named() {
        let text = super::render_template(&task()).replace("priority = 2", "priority = \"high\"");
        let err = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap_err();
        assert!(err.starts_with("line 5:"), "{err}");
        assert!(err.contains("priority"));
    }

    #[test]
    fn broken_toml_names_its_line() {
        let text = super::render_template(&task()).replace("body = \"\"", "body = \"unterminated");
        let err = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap_err();
        assert!(err.starts_with("line 4:"), "{err}");
    }
}
