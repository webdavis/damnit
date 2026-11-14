use std::io::Write;

use dam_domain::{Object, Timestamp, When};
use jiff::tz::TimeZone;

use crate::error::CliError;
use crate::layout::{fit, terminal_width};
use crate::paint::{Tint, painted};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Human,
    Json,
    Toon,
}

impl Format {
    pub(crate) fn of(json: bool, toon: bool) -> Format {
        if json {
            Format::Json
        } else if toon {
            Format::Toon
        } else {
            Format::Human
        }
    }
}

#[derive(Debug)]
pub(crate) struct Report {
    pub(crate) human: String,
    pub(crate) data: serde_json::Value,
    pub(crate) cut: Vec<bool>,
    pub(crate) tints: Vec<Tint>,
    pub(crate) stderr: Vec<String>,
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
                tints: Vec::new(),
                stderr: Vec::new(),
            };
            render(format, &report).unwrap_or_else(|_| format!("dam: {error}"))
        }
    };
    eprintln!("{line}");
}

pub(crate) fn print(format: Format, color: bool, report: &Report) -> Result<(), CliError> {
    let terminal =
        terminal_size::terminal_size_of(std::io::stdout()).map(|(w, _)| usize::from(w.0));
    let width = terminal_width(std::env::var("COLUMNS").ok().as_deref(), terminal);
    let text = match (format, width) {
        (Format::Human, Some(width)) => fitted(report, width),
        _ => render(format, report)?,
    };
    let text = match format {
        Format::Human if color => painted(&text, &report.tints),
        _ => text,
    };
    if format == Format::Human {
        for line in &report.stderr {
            eprintln!("{line}");
        }
    }
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
mod tests;
