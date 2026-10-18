use std::time::Duration;

use dam_adapters::parse_duration;
use dam_application::RemoteName;
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

pub(super) fn max_age_arg(text: &str) -> Result<(RemoteName, Duration), String> {
    let (remote, age) = text
        .split_once('=')
        .ok_or("expected <remote>=<number><s|m|h>")?;
    let remote = RemoteName::parse(remote).map_err(|e| e.to_string())?;
    let age = parse_duration("--max-age", age)
        .map_err(|_| format!("{age:?} is not a duration such as 90s, 15m or 2h"))?;
    Ok((remote, age))
}

#[cfg(test)]
mod tests;
