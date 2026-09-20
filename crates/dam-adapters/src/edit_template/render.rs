use dam_domain::{Object, When};
use toml::Value;

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
