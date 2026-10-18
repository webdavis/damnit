use dam_domain::{Path, PathError};

pub(super) fn path_arg(text: &str) -> Result<Path, String> {
    Path::parse(text).map_err(|e| match e {
        PathError::EmptySegment => "a path has an empty segment".to_string(),
        other => other.to_string(),
    })
}

#[cfg(test)]
mod tests;
