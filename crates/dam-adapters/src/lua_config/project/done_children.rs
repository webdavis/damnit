use std::path::Path;

use dam_application::{ChildDisposition, ConfigError, ProjectRuleFault};

use super::super::strict::StrictMap;

pub(super) fn done_children(
    file: &Path,
    root: &StrictMap,
) -> Result<Option<ChildDisposition>, ConfigError> {
    match root.string("done_children") {
        Ok(None) => Ok(None),
        Ok(Some(text)) => ChildDisposition::parse(&text)
            .map(Some)
            .ok_or_else(|| misshapen(file)),
        Err(_) => Err(misshapen(file)),
    }
}

fn misshapen(file: &Path) -> ConfigError {
    ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "done_children",
        fault: ProjectRuleFault::Shape,
    }
}
