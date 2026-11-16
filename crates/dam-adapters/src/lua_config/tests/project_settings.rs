use dam_application::{ConfigError, Misread, Place, ProjectSettings};

use super::project_support::ProjectFolder;

fn top_of_the_project_file() -> Place {
    Place {
        file: ".dam.lua".into(),
        table: None,
    }
}

#[test]
fn the_path_setting_is_read() {
    assert_eq!(
        ProjectFolder::with("return { path = 'damnit/' }").read(),
        Ok(ProjectSettings {
            path: "damnit/".into()
        })
    );
}

#[test]
fn a_file_without_a_path_is_refused_naming_it() {
    assert_eq!(
        ProjectFolder::with("return {}").read(),
        Err(Misread::Missing {
            key: "path".into(),
            at: top_of_the_project_file(),
        }
        .into())
    );
}

#[test]
fn any_other_key_is_refused_by_name() {
    assert_eq!(
        ProjectFolder::with("return { path = 'a', colour = 'red' }").read(),
        Err(Misread::UnknownKey {
            key: "colour".into(),
            at: top_of_the_project_file(),
        }
        .into())
    );
}

#[test]
fn a_key_that_belongs_to_a_person_is_refused_as_personal() {
    for key in [
        "remote",
        "token",
        "owner",
        "schema",
        "project_hooks",
        "hook",
        "secrets",
        "sync",
        "except",
    ] {
        let folder = ProjectFolder::with(&format!("return {{ path = 'a', {key} = 1 }}"));
        assert_eq!(
            folder.read(),
            Err(ConfigError::BelongsInPersonalConfig {
                key: key.into(),
                file: folder.file(),
            })
        );
    }
}
