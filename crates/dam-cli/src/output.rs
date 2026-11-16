use std::io::Write;

use crate::error::CliError;
use crate::layout::{fit, terminal_width};
use crate::paint::{Tint, painted};

#[derive(Debug)]
pub(crate) struct Report {
    pub(crate) human: String,
    pub(crate) data: serde_json::Value,
    pub(crate) cut: Vec<bool>,
    pub(crate) tints: Vec<Tint>,
    pub(crate) colored: Option<String>,
    pub(crate) standard_error_lines: Vec<String>,
}

impl Report {
    pub(crate) fn nothing() -> Report {
        Report {
            human: String::new(),
            data: serde_json::Value::Null,
            cut: Vec::new(),
            tints: Vec::new(),
            colored: None,
            standard_error_lines: Vec::new(),
        }
    }
}

pub(crate) fn render(format: Format, report: &Report) -> Result<String, CliError> {
    match format {
        Format::Human => Ok(report.human.clone()),
        Format::Quiet => Ok(String::new()),
        Format::Json => serde_json::to_string_pretty(&report.data)
            .map_err(|e| CliError::Internal(e.to_string())),
        Format::Toon => {
            toon_format::encode_default(&report.data).map_err(|e| CliError::Internal(e.to_string()))
        }
        Format::Yaml => Ok(yaml::encode(&report.data)),
    }
}

pub(crate) fn print_error(format: Format, error: &CliError) {
    if let CliError::Reported = error {
        return;
    }
    let line = match format {
        Format::Human | Format::Quiet => format!("dam: {error}"),
        Format::Json | Format::Toon | Format::Yaml => {
            let report = Report {
                human: String::new(),
                data: error.document(),
                cut: Vec::new(),
                tints: Vec::new(),
                colored: None,
                standard_error_lines: Vec::new(),
            };
            render(format, &report)
                .map(|text| text.trim_end().to_string())
                .unwrap_or_else(|_| format!("dam: {error}"))
        }
    };
    eprintln!("{line}");
}

pub(crate) fn print(format: Format, color: bool, report: &Report) -> Result<(), CliError> {
    let terminal =
        terminal_size::terminal_size_of(std::io::stdout()).map(|(w, _)| usize::from(w.0));
    let width = terminal_width(std::env::var("COLUMNS").ok().as_deref(), terminal);
    let Some(text) = shown(format, color, width, report)? else {
        return Ok(());
    };
    if format == Format::Human {
        for line in &report.standard_error_lines {
            eprintln!("{line}");
        }
    }
    let mut out = std::io::stdout().lock();
    let written = out.write_all(text.as_bytes()).and_then(|()| {
        match text.is_empty() || text.ends_with('\n') {
            true => Ok(()),
            false => out.write_all(b"\n"),
        }
    });
    match written {
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => Err(e.into()),
        _ => Ok(()),
    }
}

pub(crate) fn shown(
    format: Format,
    color: bool,
    width: Option<usize>,
    report: &Report,
) -> Result<Option<String>, CliError> {
    if format == Format::Quiet || report.data.is_null() {
        return Ok(None);
    }
    let text = match (format, width) {
        (Format::Human, Some(width)) => fitted(report, width),
        _ => render(format, report)?,
    };
    let text = match format {
        Format::Human if color => match &report.colored {
            Some(colored) => colored.clone(),
            None => painted(&text, &report.tints),
        },
        _ => text,
    };
    Ok(Some(text))
}

pub(crate) fn fitted(report: &Report, width: usize) -> String {
    fit(&report.human, width, &report.cut)
}

mod cells;
mod format;
pub(crate) mod yaml;

pub(crate) use cells::{local_time, object_cells};
pub(crate) use format::Format;

#[cfg(test)]
mod tests;
