use dam_application::{ConfigError, ProjectRuleFault};

use super::project_support::ProjectFolder;

#[test]
fn a_file_that_needs_a_newer_dam_is_refused_before_any_other_key_is_read() {
    let folder = ProjectFolder::with("return { minimum_dam = '9.0.0', rule_from_the_future = 1 }");
    assert_eq!(
        folder.read_as("0.2.0"),
        Err(ConfigError::NeedsANewerDam {
            file: folder.file(),
            needs: "9.0.0".into(),
            this: "0.2.0".into(),
        })
    );
}

#[test]
fn a_file_this_dam_meets_is_read() {
    for needs in ["0.2.0", "0.1.9", "0.0.10"] {
        let folder = ProjectFolder::with(&format!(
            "return {{ minimum_dam = '{needs}', path = 'a/' }}"
        ));
        assert!(folder.read_as("0.2.0").is_ok(), "{needs}");
    }
    let folder = ProjectFolder::with("return { minimum_dam = '0.10.0', path = 'a/' }");
    assert!(folder.read_as("0.9.0").is_err());
}

#[test]
fn a_minimum_that_is_not_a_version_is_refused_without_quoting_it() {
    for text in ["'latest'", "'1.2'", "3", "'1.2.3.4'"] {
        let folder =
            ProjectFolder::with(&format!("return {{ minimum_dam = {text}, path = 'a/' }}"));
        assert_eq!(
            folder.read_as("0.2.0"),
            Err(ConfigError::ProjectRule {
                file: folder.file(),
                key: "minimum_dam",
                fault: ProjectRuleFault::Shape,
            }),
            "{text}"
        );
    }
}
