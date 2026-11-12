mod support;

use std::os::unix::fs::PermissionsExt;

use support::sandbox::Sandbox;

const CREATES: usize = 20;

#[test]
fn creates_run_at_once_each_keep_their_token() {
    let _guard = support::guard("creates_run_at_once_each_keep_their_token");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
    let names: Vec<String> = (0..CREATES).map(|i| format!("c{i}")).collect();
    let running: Vec<_> = names
        .iter()
        .map(|name| sb.spawn(&["token", "create", name, "--allow", "task:read"]))
        .collect();
    let values: Vec<String> = running
        .into_iter()
        .map(|child| {
            let out = child.wait_with_output().unwrap();
            assert!(out.status.success(), "{out:?}");
            String::from_utf8(out.stdout)
                .unwrap()
                .trim_end()
                .to_string()
        })
        .collect();
    let config = std::fs::read_to_string(&config).unwrap();
    for (name, value) in names.iter().zip(&values) {
        assert!(config.contains(&format!("[token.{name}]")), "{config}");
        let out = sb.output_holding(value, &["ls", "--json", "--no-pull"]);
        assert!(out.status.success(), "{name}: {out:?}");
    }
}
