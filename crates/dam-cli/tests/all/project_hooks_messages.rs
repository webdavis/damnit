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

#[test]
fn a_projects_hook_failure_reaches_the_terminal_without_a_control_character() {
    let _guard =
        support::guard("a_projects_hook_failure_reaches_the_terminal_without_a_control_character");
    let sb = Sandbox::new();
    let app = sb.dir.path().join("app");
    std::fs::create_dir_all(app.join(".damhooks")).unwrap();
    std::fs::write(app.join(".dam.lua"), "return { path = 'app/' }\n").unwrap();
    std::fs::write(
        app.join(".damhooks/task_new.lua"),
        "return function() error('\\27[2Jgone\\rhere') end\n",
    )
    .unwrap();
    let run = |args: &[&str]| sb.command(args).current_dir(&app).output().unwrap();
    assert!(
        run(&["config", "set", "project.hooks", "true"])
            .status
            .success()
    );
    for args in [
        &["new", "water the plants"][..],
        &["hook", "run", "task_new"],
    ] {
        let out = run(args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "{err}");
        assert!(err.contains("gone\u{fffd}here"), "{args:?}: {err:?}");
        assert!(
            !err.contains('\u{1b}') && !err.contains('\r'),
            "{args:?}: {err:?}"
        );
    }
}

#[test]
fn hook_list_never_prints_a_damhooks_name_holding_a_control_character() {
    let _guard =
        support::guard("hook_list_never_prints_a_damhooks_name_holding_a_control_character");
    let sb = Sandbox::new();
    let app = sb.dir.path().join("app");
    std::fs::create_dir_all(app.join(".damhooks")).unwrap();
    std::fs::write(app.join(".dam.lua"), "return { path = 'app/' }\n").unwrap();
    for name in ["task_new\u{1b}[2J.lua", "task_done\r.lua"] {
        std::fs::write(app.join(".damhooks").join(name), "return function() end\n").unwrap();
    }
    let run = |args: &[&str]| sb.command(args).current_dir(&app).output().unwrap();
    for on in ["false", "true"] {
        assert!(
            run(&["config", "set", "project.hooks", on])
                .status
                .success()
        );
        for args in [&["hook", "list"][..], &["hook", "list", "--json"]] {
            let out = run(args);
            let printed = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(out.status.success(), "{printed}");
            assert!(
                !printed.contains('\u{1b}') && !printed.contains('\r'),
                "{printed:?}"
            );
            if args.len() == 2 {
                assert!(printed.contains("2 files in .damhooks"), "{printed}");
            }
        }
        let json: serde_json::Value =
            serde_json::from_slice(&run(&["hook", "list", "--json"]).stdout).unwrap();
        assert_eq!(json["unshown_files"], 2);
        support::schema::assert_matches_schema(&json, "hook-list.schema.json");
    }
}
