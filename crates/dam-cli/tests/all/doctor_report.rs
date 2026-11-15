use crate::support;

use support::sandbox::Sandbox;

#[test]
fn a_broken_config_leaves_only_the_database_to_check() {
    let _guard = support::guard("a_broken_config_leaves_only_the_database_to_check");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    let answer = sb.doctor(&["doctor"]);
    assert_eq!(answer.code, Some(1));
    let rows: Vec<&str> = answer.out.lines().skip(5).take(5).collect();
    assert_eq!(
        rows,
        [
            "  ✓ database: none yet",
            "    → dam doctor --fix creates it",
            "  · helpers: not checked, the config did not read",
            "  · credentials: not checked, the config did not read",
            "  · pushes: not checked, the config did not read",
        ]
    );
    assert!(!sb.dir.path().join("token.txt").exists());
}
