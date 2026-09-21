//! The words `done --children` and `done --depends` accept, one per answer
//! the completion prompt offers.

use dam_domain::{ChildDisposition, DependencyDisposition};

/// `--children`: the words for the three answers the prompt offers, with the
/// group's name carried inline so the flag needs no second value.
pub(super) fn child_disposition(text: &str) -> Result<ChildDisposition, String> {
    match text.split_once(':') {
        Some(("into", name)) if !name.is_empty() => Ok(ChildDisposition::Into(name.to_string())),
        _ => match text {
            "up" => Ok(ChildDisposition::Up),
            "keep" => Ok(ChildDisposition::Keep),
            _ => Err("one of up, keep, into:<name>".to_string()),
        },
    }
}

pub(super) fn dependency_disposition(text: &str) -> Result<DependencyDisposition, String> {
    match text {
        "drop" => Ok(DependencyDisposition::Drop),
        "keep" => Ok(DependencyDisposition::Keep),
        _ => Err("one of drop, keep".to_string()),
    }
}
