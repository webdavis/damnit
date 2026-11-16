use crate::support;

use support::sandbox::Sandbox;

#[test]
fn a_failing_projects_hook_names_its_file_under_damhooks() {
    let _guard = support::guard("a_failing_projects_hook_names_its_file_under_damhooks");
    let sb = Sandbox::new();
    let app = sb.dir.path().join("app");
    std::fs::create_dir_all(app.join(".damhooks")).unwrap();
    std::fs::write(app.join(".dam.lua"), "return { path = 'app/' }\n").unwrap();
    std::fs::write(
        app.join(".damhooks/task_new.lua"),
        "return function() error('no water') end\n",
    )
    .unwrap();
    let on = sb
        .command(&["config", "set", "project.hooks", "true"])
        .current_dir(&app)
        .output();
    assert!(on.unwrap().status.success());
    let out = sb
        .command(&["new", "water the plants"])
        .current_dir(&app)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{err}");
    assert!(err.contains("app/.damhooks/task_new.lua"), "{err}");
    assert!(err.contains("no water"), "{err}");
}
