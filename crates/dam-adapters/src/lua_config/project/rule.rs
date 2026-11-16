use std::path::Path;

use dam_application::{ConfigError, ProjectRuleFault};

pub(super) fn misshapen(file: &Path, key: &'static str) -> impl Fn(ConfigError) -> ConfigError {
    let file = file.to_path_buf();
    move |_| ConfigError::ProjectRule {
        file: file.clone(),
        key,
        fault: ProjectRuleFault::Shape,
    }
}
