use std::io::Write;

use dam_domain::{Object, Timestamp, When};
use jiff::tz::TimeZone;

use crate::error::CliError;
use crate::layout::{fit, terminal_width};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Human,
    Json,
    Toon,
}

#[derive(Debug)]
pub(crate) struct Report {
    pub(crate) human: String,
    pub(crate) data: serde_json::Value,
    pub(crate) cut: Vec<bool>,
}

pub(crate) fn render(format: Format, report: &Report) -> Result<String, CliError> {
    match format {
        Format::Human => Ok(report.human.clone()),
        Format::Json => serde_json::to_string_pretty(&report.data)
            .map_err(|e| CliError::Internal(e.to_string())),
        Format::Toon => {
            toon_format::encode_default(&report.data).map_err(|e| CliError::Internal(e.to_string()))
        }
    }
}

pub(crate) fn print_error(format: Format, error: &CliError) {
    let line = match format {
        Format::Human => format!("dam: {error}"),
        Format::Json | Format::Toon => {
            let report = Report {
                human: String::new(),
                data: error.document(),
                cut: Vec::new(),
            };
            render(format, &report).unwrap_or_else(|_| format!("dam: {error}"))
        }
    };
    eprintln!("{line}");
}

pub(crate) fn print(format: Format, report: &Report) -> Result<(), CliError> {
    let terminal =
        terminal_size::terminal_size_of(std::io::stdout()).map(|(w, _)| usize::from(w.0));
    let width = terminal_width(std::env::var("COLUMNS").ok().as_deref(), terminal);
    let text = match (format, width) {
        (Format::Human, Some(width)) => fitted(report, width),
        _ => render(format, report)?,
    };
    let mut out = std::io::stdout().lock();
    out.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
}

pub(crate) fn fitted(report: &Report, width: usize) -> String {
    fit(&report.human, width, &report.cut)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::{ObjectRow, report};
    use dam_domain::{Oid, Path, Priority, Task, When};
    use jiff::civil::date;

    #[test]
    fn an_object_row_has_short_oid_priority_due_path_then_subject_and_labels() {
        let mut t = Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(0x3f)),
            "buy oat milk",
        );
        t.priority = Priority::new(2).unwrap();
        t.due = Some(When::Day(date(2026, 9, 25)));
        t.base.labels.insert("errand".into());
        t.base.path = Path::parse("work").unwrap();
        let line = report(&ObjectRow(&Object::Task(t), &TimeZone::UTC))
            .unwrap()
            .human;
        assert_eq!(
            line,
            "3f3f3f3  p2  2026-09-25  work/  buy oat milk  [errand]"
        );
    }

    #[test]
    fn a_timed_event_reads_in_local_time() {
        let at = "2026-10-13T14:00:00+00:00[UTC]".parse().unwrap();
        let zone = TimeZone::fixed(jiff::tz::offset(-4));
        assert_eq!(local_text(&When::At(at), &zone), "2026-10-13 10:00");
    }

    #[test]
    fn toon_renders_a_list_as_a_table() {
        let data = serde_json::json!({"objects": [{"oid": "a", "subject": "x"}, {"oid": "b", "subject": "y"}]});
        let text = render(
            Format::Toon,
            &Report {
                human: String::new(),
                data,
                cut: Vec::new(),
            },
        )
        .unwrap();
        assert!(text.starts_with("objects[2]{oid,subject}:"), "{text}");
    }

    #[test]
    fn json_renders_the_data_and_human_renders_the_text() {
        let report = Report {
            human: "hi".into(),
            data: serde_json::json!({"a": 1}),
            cut: Vec::new(),
        };
        assert_eq!(render(Format::Human, &report).unwrap(), "hi");
        assert_eq!(render(Format::Json, &report).unwrap(), "{\n  \"a\": 1\n}");
    }
}
