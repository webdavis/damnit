use std::time::Duration;

use dam_application::{ConfiguredDuration, RemoteName};
use dam_domain::{CallerId, Kind, Path, PathError, Priority, parse_minutes};

pub(super) fn path_arg(text: &str) -> Result<Path, String> {
    Path::parse(text).map_err(|e| match e {
        PathError::EmptySegment => "a path has an empty segment".to_string(),
        other => other.to_string(),
    })
}

pub(super) fn kind_arg(text: &str) -> Result<Kind, String> {
    Kind::parse(text).ok_or_else(|| "kind is task or event".to_string())
}

pub(super) fn caller_id_arg(text: &str) -> Result<CallerId, String> {
    CallerId::parse(text).map_err(|e| e.to_string())
}

pub(super) fn priority_arg(text: &str) -> Result<Priority, String> {
    text.parse()
        .ok()
        .and_then(|n| Priority::new(n).ok())
        .ok_or_else(|| "priority is 1 (highest) to 4".to_string())
}

pub(super) fn estimate_arg(text: &str) -> Result<u32, String> {
    parse_minutes(text).ok_or_else(|| format!("{text:?} is not whole minutes such as 5, 5m or 2h"))
}

pub(super) fn max_age_arg(text: &str) -> Result<(RemoteName, Duration), String> {
    let (remote, age) = text
        .split_once('=')
        .ok_or("expected <remote>=<number><s|m|h>")?;
    let remote = RemoteName::parse(remote).map_err(|e| e.to_string())?;
    let age = ConfiguredDuration::parse(age)
        .map(|d| d.value)
        .ok_or_else(|| format!("{age:?} is not a duration such as 90s, 15m or 2h"))?;
    Ok((remote, age))
}

#[cfg(test)]
mod tests;
