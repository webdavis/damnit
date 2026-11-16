use std::path::Path;

use dam_application::{ConfigError, ProjectRuleFault};
use dam_domain::{Categories, CategoryError};

use super::super::settings::read_category;
use super::super::strict::StrictMap;

pub(super) fn categories(file: &Path, root: &StrictMap) -> Result<Categories, ConfigError> {
    let refused = |fault| ConfigError::ProjectRule {
        file: file.to_path_buf(),
        key: "category",
        fault,
    };
    let named = root
        .maps_in("category")
        .map_err(|_| refused(ProjectRuleFault::Shape))?;
    let read = named
        .iter()
        .map(|(name, category)| read_category(name, category))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| refused(ProjectRuleFault::Shape))?;
    Categories::new(read).map_err(|error| match error {
        CategoryError::DuplicateValue { .. } => refused(ProjectRuleFault::LabelInTwoCategories),
        _ => refused(ProjectRuleFault::Shape),
    })
}
