use dam_domain::{Object, Timestamp, When};
use jiff::tz::TimeZone;

pub(crate) fn object_cells(object: &Object, zone: &TimeZone) -> Vec<String> {
    let b = object.base();
    let (state, when) = match object {
        Object::Task(t) if t.completion.is_done() => ("done".to_string(), t.due.as_ref()),
        Object::Task(t) => (format!("p{}", t.priority.get()), t.due.as_ref()),
        Object::Event(e) => ("event".to_string(), Some(&e.start)),
    };
    let mut free_text = b.subject.clone();
    if !b.labels.is_empty() {
        let labels: Vec<&str> = b.labels.iter().map(String::as_str).collect();
        free_text.push_str(&format!("  [{}]", labels.join(", ")));
    }
    vec![
        b.oid.short().to_string(),
        state,
        when.map(|w| local_text(w, zone)).unwrap_or_default(),
        b.path.as_str().to_string(),
        free_text,
    ]
}

pub(crate) fn local_text(when: &When, zone: &TimeZone) -> String {
    match when {
        When::Day(d) => d.to_string(),
        When::At(z) => local_time(z.timestamp(), zone),
    }
}

pub(crate) fn local_time(at: Timestamp, zone: &TimeZone) -> String {
    at.to_zoned(zone.clone())
        .strftime("%Y-%m-%d %H:%M")
        .to_string()
}
