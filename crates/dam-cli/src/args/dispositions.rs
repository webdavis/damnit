use dam_domain::{ChildDisposition, DependencyDisposition};

pub(super) fn child_disposition(text: &str) -> Result<ChildDisposition, String> {
    ChildDisposition::parse(text).ok_or_else(|| "one of up, keep, into:<name>".to_string())
}

pub(super) fn dependency_disposition(text: &str) -> Result<DependencyDisposition, String> {
    match text {
        "drop" => Ok(DependencyDisposition::Drop),
        "keep" => Ok(DependencyDisposition::Keep),
        _ => Err("one of drop, keep".to_string()),
    }
}
