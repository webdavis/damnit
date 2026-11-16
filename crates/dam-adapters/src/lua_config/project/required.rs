use std::path::Path;

use dam_application::{ConfigError, ProjectRuleFault, RequiredField};

use super::super::strict::StrictMap;

pub(super) fn required(file: &Path, root: &StrictMap) -> Result<Vec<RequiredField>, ConfigError> {
    let misshapen = || ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "required",
        fault: ProjectRuleFault::Shape,
    };
    let named = root.strings("required").map_err(|_| misshapen())?;
    named
        .unwrap_or_default()
        .iter()
        .map(|name| field(name).ok_or_else(misshapen))
        .collect()
}

fn field(name: &str) -> Option<RequiredField> {
    [
        RequiredField::Due,
        RequiredField::Labels,
        RequiredField::Priority,
        RequiredField::Body,
        RequiredField::Deadline,
    ]
    .into_iter()
    .find(|field| field.name() == name)
}
