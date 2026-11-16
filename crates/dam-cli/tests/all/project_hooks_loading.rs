use std::path::PathBuf;

use crate::support;

use support::sandbox::Sandbox;

const EVENTS: [&str; 4] = ["task_new", "task_changed", "task_done", "task_pulled"];

fn project(sb: &Sandbox, name: &str) -> PathBuf {
    let folder = sb.dir.path().join(name);
    std::fs::create_dir_all(folder.join(".damhooks")).unwrap();
    std::fs::write(
        folder.join(".dam.lua"),
        format!("return {{ path = '{name}/' }}\n"),
    )
    .unwrap();
    for event in EVENTS {
        let hook = format!(
            "local f = assert(io.open(os.getenv('HOME') .. '/loaded-{name}', 'a'))\n\
             f:write('{event}\\n')\nf:close()\n\
             return function() return {{ labels = {{ 'from-{name}' }} }} end\n"
        );
        std::fs::write(folder.join(".damhooks").join(format!("{event}.lua")), hook).unwrap();
    }
    folder
}

fn dam_in(sb: &Sandbox, folder: &PathBuf, args: &[&str]) -> String {
    let out = sb.command(args).current_dir(folder).output().unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn created(sb: &Sandbox, folder: &PathBuf, args: &[&str]) -> serde_json::Value {
    let mut line = vec!["new", "--json"];
    line.extend_from_slice(args);
    serde_json::from_str(&dam_in(sb, folder, &line)).unwrap()
}

fn loaded(sb: &Sandbox, name: &str) -> bool {
    sb.dir.path().join(format!("loaded-{name}")).exists()
}

#[test]
fn with_project_hooks_off_no_command_so_much_as_loads_a_hook_file() {
    let _guard = support::guard("with_project_hooks_off_no_command_so_much_as_loads_a_hook_file");
    let sb = Sandbox::new();
    let app = project(&sb, "app");
    let task = created(&sb, &app, &["water the plants"]);
    let oid = task["oid"].as_str().unwrap();
    assert_eq!(task["labels"], serde_json::json!([]));
    dam_in(&sb, &app, &["edit", oid, "--subject", "water the ferns"]);
    dam_in(&sb, &app, &["done", oid]);
    dam_in(&sb, &app, &["hook", "list"]);
    dam_in(&sb, &app, &["config", "list"]);
    assert!(!loaded(&sb, "app"));
}

#[test]
fn turned_on_for_this_clone_the_projects_hook_runs_and_no_other_projects_loads() {
    let _guard = support::guard(
        "turned_on_for_this_clone_the_projects_hook_runs_and_no_other_projects_loads",
    );
    let sb = Sandbox::new();
    let app = project(&sb, "app");
    let other = project(&sb, "other");
    dam_in(&sb, &app, &["config", "set", "project.hooks", "true"]);
    let task = created(&sb, &app, &["water the plants"]);
    assert_eq!(task["labels"], serde_json::json!(["from-app"]));
    assert!(loaded(&sb, "app"));
    let elsewhere = created(&sb, &other, &["feed the cat"]);
    assert_eq!(elsewhere["labels"], serde_json::json!([]));
    assert!(!loaded(&sb, "other"));
}

#[test]
fn global_and_project_flags_run_no_projects_hook() {
    let _guard = support::guard("global_and_project_flags_run_no_projects_hook");
    let sb = Sandbox::new();
    let app = project(&sb, "app");
    dam_in(
        &sb,
        &app,
        &["config", "set", "--global", "project.hooks", "true"],
    );
    let global = created(&sb, &app, &["water the plants", "--global"]);
    let elsewhere = created(&sb, &app, &["water the plants", "--project", "app/"]);
    assert_eq!(global["labels"], serde_json::json!([]));
    assert_eq!(elsewhere["labels"], serde_json::json!([]));
    assert!(!loaded(&sb, "app"));
}
