use dam_application::{ConfigError, ProjectRuleFault};

use super::project_support::ProjectFolder;

#[test]
fn standard_sections_are_read() {
    let folder = ProjectFolder::with("return { path = 'a/', sections = { 'bugs', 'features' } }");
    assert_eq!(folder.read().unwrap().sections, ["bugs", "features"]);
}

#[test]
fn a_section_that_is_not_one_path_segment_is_refused_naming_only_the_file() {
    for text in [
        "return { path = 'a/', sections = 'bugs' }",
        "return { path = 'a/', sections = { 'bugs/old' } }",
        "return { path = 'a/', sections = { '' } }",
        "return { path = 'a/', sections = { 'bugs', 'bugs' } }",
        "return { path = 'a/', sections = { 1 } }",
    ] {
        let folder = ProjectFolder::with(text);
        assert_eq!(
            folder.read(),
            Err(ConfigError::ProjectRule {
                file: folder.file(),
                key: "sections",
                fault: ProjectRuleFault::Shape,
            }),
            "{text}"
        );
    }
}
