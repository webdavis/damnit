use serde_json::json;

use crate::output::Format;

const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Clone, Copy, Debug)]
pub(crate) struct Moment<'a> {
    pub(crate) name: &'a str,
    pub(crate) remote: Option<&'a str>,
    pub(crate) doing: &'a str,
    pub(crate) did: &'a str,
    pub(crate) unit: &'a str,
    pub(crate) done: usize,
    pub(crate) total: Option<usize>,
    pub(crate) seconds: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Stage {
    Started,
    Running(usize),
    Finished,
    Failed,
}

pub(crate) fn line(moment: &Moment<'_>, stage: Stage, format: Format) -> String {
    match format {
        Format::Human | Format::Quiet => text(moment, stage),
        Format::Json => record(moment, stage).to_string(),
        Format::Toon => toon_format::encode_default(&record(moment, stage)).unwrap_or_default(),
    }
}

fn text(moment: &Moment<'_>, stage: Stage) -> String {
    let count = match moment.total {
        _ if moment.unit.is_empty() => String::new(),
        Some(total) => format!(
            "  {} of {} {}",
            grouped(moment.done),
            grouped(total),
            moment.unit
        ),
        None => format!("  {} {}", grouped(moment.done), moment.unit),
    };
    let seconds = moment.seconds;
    match stage {
        Stage::Started => format!("{}{count}  {seconds}s", moment.doing),
        Stage::Running(frame) => format!(
            "{} {}{count}  {seconds}s",
            SPINNER[frame % SPINNER.len()],
            moment.doing
        ),
        Stage::Finished => format!("✓ {}{count}  {seconds}s", moment.did),
        Stage::Failed => format!("✗ {} failed{count}  {seconds}s", moment.doing),
    }
}

fn record(moment: &Moment<'_>, stage: Stage) -> serde_json::Value {
    let stage = match stage {
        Stage::Finished => "finished",
        Stage::Failed => "failed",
        Stage::Started | Stage::Running(_) => "started",
    };
    let mut record = json!({
        "step": moment.name,
        "stage": stage,
        "done": moment.done,
        "total": moment.total,
        "seconds": moment.seconds,
    });
    if let Some(remote) = moment.remote {
        record["remote"] = json!(remote);
    }
    record
}

fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests;
