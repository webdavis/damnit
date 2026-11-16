use dam_adapters::LuaConfigFile;
use dam_adapters::ProjectHookFolder;
use dam_application::testing::FakeProjectHookSwitches;
use dam_application::{Project, ProjectHookSwitch, ProjectHookSwitches, ProjectSettings};

use super::{ProjectHookLook, run};
use crate::output::{Format, Report, shown};
use crate::project_scope::Standing;

fn listed_with(files: &[(&str, &str)], on: bool) -> Report {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join(".damhooks")).unwrap();
    for (name, text) in files {
        std::fs::write(app.join(".damhooks").join(name), text).unwrap();
    }
    let switches = FakeProjectHookSwitches::default();
    let switch = match on {
        true => ProjectHookSwitch::on_for(
            ProjectHookFolder::read(&app)
                .unwrap()
                .fingerprint()
                .unwrap()
                .unwrap(),
        ),
        false => ProjectHookSwitch::off(),
    };
    switches.turn(&app, Some(switch)).unwrap();
    let standing = Standing::Inside(Box::new(Project {
        folder: app,
        settings: ProjectSettings {
            path: "app/".into(),
            ..Default::default()
        },
    }));
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    run(&file, &ProjectHookLook::of(&standing, &switches)).unwrap()
}

fn everything_printed(report: &Report) -> String {
    let json = shown(Format::Json, false, None, report).unwrap();
    format!(
        "{}\n{}\n{}",
        report.human,
        json.unwrap_or_default(),
        report.standard_error_lines.join("\n")
    )
}

const ODD_NAMES: [(&str, &str); 2] = [
    ("task_new\u{1b}[2J.lua", "return function() end"),
    ("task_done\r.lua", "return function() end"),
];

#[test]
fn names_holding_a_control_character_are_counted_and_never_printed() {
    for on in [false, true] {
        let report = listed_with(&ODD_NAMES, on);
        let printed = everything_printed(&report);
        assert!(
            !printed.chars().any(|c| c.is_control() && c != '\n'),
            "{printed:?}"
        );
        assert!(!printed.contains("[2J"), "{printed}");
        assert_eq!(report.data["unshown_files"], 2);
        assert!(
            report
                .standard_error_lines
                .contains(&"warning: 2 files in .damhooks are links or have names dam will not show; dam does not load them".into()),
            "{:?}",
            report.standard_error_lines
        );
    }
}

#[test]
fn one_odd_name_is_told_as_one_file() {
    let report = listed_with(&ODD_NAMES[..1], false);
    assert_eq!(report.data["unshown_files"], 1);
    assert!(
        report
            .standard_error_lines
            .contains(&"warning: 1 file in .damhooks is a link or has a name dam will not show; dam does not load it".into())
    );
}

#[test]
fn a_projects_hook_that_does_not_load_warns_without_what_it_raised() {
    let report = listed_with(&[("task_new.lua", "error('\\27[2J gone')")], true);
    let printed = everything_printed(&report);
    assert!(!printed.contains("[2J"), "{printed}");
    assert!(report.human.contains("does not load"), "{}", report.human);
}
