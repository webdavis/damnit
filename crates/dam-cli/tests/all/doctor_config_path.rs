use crate::support;

use support::sandbox::Sandbox;

fn through_a_toml_config(sb: &Sandbox, args: &[&str]) -> (Option<i32>, String) {
    let config = sb.dir.path().join("config.toml");
    let out = sb
        .command(args)
        .env("DAM_CONFIG", &config)
        .output()
        .unwrap();
    assert!(out.stderr.is_empty(), "{args:?}: {out:?}");
    (out.status.code(), String::from_utf8(out.stdout).unwrap())
}

#[test]
fn a_config_path_that_does_not_end_in_lua_is_an_issue_in_the_report() {
    let _guard = support::guard("a_config_path_that_does_not_end_in_lua_is_an_issue_in_the_report");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    let detail = format!(
        "config: {} does not end in .lua; the config is config.lua now",
        config.display()
    );
    let (code, out) = through_a_toml_config(&sb, &["doctor"]);
    assert_eq!(code, Some(1));
    let rows =
        format!("\n  ✗ {detail}\n    → point --config or DAM_CONFIG at a file that ends in .lua\n");
    assert!(out.contains(&rows), "{out}");
    assert!(out.contains(&format!("\n  1. {detail}\n")), "{out}");
}
