use dam_domain::{Path, PathError, Priority};

pub(super) fn path_arg(text: &str) -> Result<Path, String> {
    Path::parse(text).map_err(|e| match e {
        PathError::EmptySegment => "a path has an empty segment".to_string(),
        other => other.to_string(),
    })
}

pub(super) fn priority_arg(text: &str) -> Result<Priority, String> {
    text.parse()
        .ok()
        .and_then(|n| Priority::new(n).ok())
        .ok_or_else(|| "priority is 1 (highest) to 4".to_string())
}

#[cfg(test)]
mod tests;
