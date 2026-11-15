use crate::support;

use support::sandbox::Sandbox;
use support::small_config::{MARKER, small_config};

fn token_value(sb: &Sandbox) -> String {
    let secrets = std::fs::read_to_string(sb.secrets_path()).unwrap();
    let start = secrets.find("damk_").unwrap();
    let rest = &secrets[start..];
    rest[..rest.find('"').unwrap()].to_string()
}

#[test]
fn no_secret_reaches_any_stream() {
    let _guard = support::guard("no_secret_reaches_any_stream");
    let sb = small_config();
    let token = token_value(&sb);
    for line in [
        &["config", "list"][..],
        &["config", "list", "--json"],
        &["config", "list", "--toon"],
        &["config", "get", "remote.todoist"],
        &["config", "get", "remote.todoist", "--json"],
        &["config", "get", "remote.todoist.credential", "--json"],
        &["config", "get", "token.tool", "--json"],
    ] {
        let out = sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        for stream in [&out.stdout, &out.stderr] {
            let text = String::from_utf8_lossy(stream);
            assert!(!text.contains(MARKER), "{line:?}: {text}");
            assert!(!text.contains(&token), "{line:?}: {text}");
        }
    }
}
