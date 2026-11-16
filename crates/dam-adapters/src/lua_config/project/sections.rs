use std::collections::BTreeSet;
use std::path::Path;

use dam_application::{ConfigError, ProjectRuleFault};

use super::super::strict::StrictMap;

pub(super) fn sections(file: &Path, root: &StrictMap) -> Result<Vec<String>, ConfigError> {
    let refused = || ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "sections",
        fault: ProjectRuleFault::Shape,
    };
    let names = root.strings("sections").map_err(|_| refused())?;
    let names = names.unwrap_or_default();
    let mut seen = BTreeSet::new();
    let segments = names
        .iter()
        .all(|name| seen.insert(name) && dam_domain::Path::root().join(name).is_ok());
    segments.then_some(names).ok_or_else(refused)
}
