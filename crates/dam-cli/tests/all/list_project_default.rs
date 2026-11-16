use crate::support;

use support::sandbox::Sandbox;

fn subjects(sb: &Sandbox, app: &std::path::Path, flags: &[&str]) -> Vec<String> {
    let line = [&["list"], flags, &["-o", "json"]].concat();
    let out = sb.command(&line).current_dir(app).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut subjects: Vec<String> = listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect();
    subjects.sort();
    subjects
}

#[test]
fn the_projects_listing_gives_way_to_a_selection_flag_and_not_to_an_output_flag() {
    let _guard = support::guard(
        "the_projects_listing_gives_way_to_a_selection_flag_and_not_to_an_output_flag",
    );
    let sb = Sandbox::new();
    let app = sb.dir.path().join("app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(
        app.join(".dam.lua"),
        "return { path = 'app/', list = { priority = 1 } }\n",
    )
    .unwrap();
    sb.new_object(&["urgent", "--path", "app/", "-p", "1"]);
    sb.new_object(&["calm", "--path", "app/cli/", "-p", "4"]);
    sb.new_object(&["elsewhere", "--path", "home/", "-p", "4"]);
    assert_eq!(subjects(&sb, &app, &[]), ["urgent"]);
    assert_eq!(subjects(&sb, &app, &["--priority", "4"]), ["calm"]);
    assert_eq!(subjects(&sb, &app, &["--path", "cli/"]), ["calm"]);
    assert_eq!(
        subjects(&sb, &app, &["--global", "--priority", "4"]),
        ["calm", "elsewhere"]
    );
}
