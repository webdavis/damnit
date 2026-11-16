use crate::nvim_fixtures::{answer, assert_fixture};
use crate::support;

use support::fixture;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn without_the_sandbox(sb: &Sandbox, text: &str) -> String {
    let real = sb.dir.path().canonicalize().unwrap();
    text.replace(&real.display().to_string(), "<sandbox>")
        .replace(&sb.dir.path().display().to_string(), "<sandbox>")
}

fn config_list_in(sb: &Sandbox, folder: &std::path::Path) -> String {
    let out = sb
        .command(&["config", "list", "-o", "json"])
        .current_dir(folder)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert_matches_schema(
        &serde_json::from_str(&text).unwrap(),
        "config-list.schema.json",
    );
    without_the_sandbox(sb, &text)
}

#[test]
fn the_config_list_fixtures_are_what_dam_answers_in_and_outside_a_project() {
    let _guard =
        support::guard("the_config_list_fixtures_are_what_dam_answers_in_and_outside_a_project");
    let sb = Sandbox::new();
    let outside = sb.dir.path().join("notes");
    std::fs::create_dir_all(&outside).unwrap();
    assert_fixture("config-list.json", &config_list_in(&sb, &outside));
    let project = sb.dir.path().join("app");
    std::fs::create_dir_all(&project).unwrap();
    let init = sb
        .command(&["init", "-o", "json"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    assert_fixture("config-list-project.json", &config_list_in(&sb, &project));
}

#[test]
fn the_list_columns_fixture_is_what_dam_list_answers_for_rows_with_every_column() {
    let _guard = support::guard(
        "the_list_columns_fixture_is_what_dam_list_answers_for_rows_with_every_column",
    );
    let sb = fixture::seeded();
    let offsite = sb.new_object(&[
        "plan the offsite",
        "--priority",
        "1",
        "--labels",
        "urgent",
        "--category",
        "effort=deep",
        "--due",
        "2026-09-14",
    ]);
    sb.new_object(&[
        "book venue",
        "--parent",
        &offsite,
        "--estimate",
        "30",
        "--due",
        "2030-11-18",
    ]);
    sb.new_object(&["send invites", "--priority", "2", "--depends", &offsite]);
    sb.new_object(&[
        "fix the login redirect loop that sends people back to log in",
        "--estimate",
        "45",
        "--deadline",
        "2030-11-20",
        "--due",
        "2030-11-16",
    ]);
    let text = answer(&sb, &["list", "-o", "json"], "list.schema.json");
    assert_fixture("list-columns.json", &text);
}
