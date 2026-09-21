use std::io::Write;

use dam_adapters::to_wire;
use dam_domain::Object;

use crate::error::CliError;

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
}

pub(crate) fn render(format: Format, report: &Report) -> Result<String, CliError> {
    match format {
        Format::Human => Ok(report.human.clone()),
        Format::Json => {
            serde_json::to_string_pretty(&report.data).map_err(|e| CliError::Io(e.to_string()))
        }
        Format::Toon => {
            toon_format::encode_default(&report.data).map_err(|e| CliError::Io(e.to_string()))
        }
    }
}

/// A failure, on standard error. A machine format prints the error document
/// so a client reads the reason; the human form keeps its plain line.
pub(crate) fn print_error(format: Format, error: &CliError) {
    let line = match format {
        Format::Human => format!("dam: {error}"),
        Format::Json | Format::Toon => {
            let report = Report {
                human: String::new(),
                data: error.document(),
            };
            render(format, &report).unwrap_or_else(|_| format!("dam: {error}"))
        }
    };
    eprintln!("{line}");
}

pub(crate) fn print(format: Format, report: &Report) -> Result<(), CliError> {
    let text = render(format, report)?;
    let mut out = std::io::stdout().lock();
    out.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
}

/// The object as the `--json` and `--toon` answers carry it. A conversion
/// failure is an error rather than a null, because a null would be a document
/// that parses and says the object has no fields.
pub(crate) fn object_json(object: &Object) -> Result<serde_json::Value, CliError> {
    serde_json::to_value(to_wire(object, None)).map_err(|e| CliError::Io(e.to_string()))
}

pub(crate) fn object_line(object: &Object) -> String {
    let b = object.base();
    let mut cols = vec![b.oid.short().to_string()];
    match object {
        Object::Task(t) => {
            cols.push(if t.done {
                "done".into()
            } else {
                format!("p{}", t.priority.get())
            });
            cols.push(
                t.due
                    .as_ref()
                    .map(|d| d.to_text())
                    .unwrap_or_else(|| "-".into()),
            );
        }
        Object::Event(e) => {
            cols.push("event".into());
            cols.push(e.start.to_text());
        }
    }
    cols.push(b.subject.clone());
    if !b.labels.is_empty() {
        cols.push(format!(
            "[{}]",
            b.labels.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if !b.path.as_str().is_empty() {
        cols.push(b.path.as_str().to_string());
    }
    cols.join("  ")
}

pub(crate) fn objects_report(objects: &[Object]) -> Result<Report, CliError> {
    let data: Vec<serde_json::Value> = objects.iter().map(object_json).collect::<Result<_, _>>()?;
    Ok(Report {
        human: objects
            .iter()
            .map(object_line)
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "objects": data }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_domain::{Oid, Path, Priority, Task, When};
    use jiff::civil::date;

    #[test]
    fn object_line_has_short_oid_priority_due_subject_labels_and_path() {
        let mut t = Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(0x3f)),
            "buy oat milk",
        );
        t.priority = Priority::new(2).unwrap();
        t.due = Some(When::Day(date(2026, 9, 25)));
        t.base.labels.insert("errand".into());
        t.base.path = Path::parse("work").unwrap();
        let line = object_line(&Object::Task(t));
        assert_eq!(
            line,
            "3f3f3f3  p2  2026-09-25  buy oat milk  [errand]  work/"
        );
    }

    #[test]
    fn toon_renders_a_list_as_a_table() {
        let data = serde_json::json!({"objects": [{"oid": "a", "subject": "x"}, {"oid": "b", "subject": "y"}]});
        let text = render(
            Format::Toon,
            &Report {
                human: String::new(),
                data,
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
        };
        assert_eq!(render(Format::Human, &report).unwrap(), "hi");
        assert_eq!(render(Format::Json, &report).unwrap(), "{\n  \"a\": 1\n}");
    }
}
