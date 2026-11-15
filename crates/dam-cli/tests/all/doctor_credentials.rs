use crate::support;

use support::sandbox::Sandbox;

fn rows(sb: &Sandbox) -> (Option<i32>, Vec<String>) {
    let answer = sb.doctor(&["doctor", "credentials"]);
    let rows = answer
        .out
        .lines()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    (answer.code, rows)
}

#[test]
fn credentials_are_not_checked_without_a_config_or_a_remote() {
    let _guard = support::guard("credentials_are_not_checked_without_a_config_or_a_remote");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    assert_eq!(
        rows(&sb).1,
        ["  · credentials: not checked, the config did not read"]
    );
    sb.write_config_lua("return {}\n");
    sb.write_secrets_lua("return { schema = 1 }\n");
    assert_eq!(rows(&sb).1, ["  · credentials: no remotes configured"]);
}
