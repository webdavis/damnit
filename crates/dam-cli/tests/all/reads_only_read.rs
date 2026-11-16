use crate::support;

use support::config_files::Remote;
use support::sandbox::Sandbox;

const READS: [&str; 4] = ["list", "show", "agenda", "reminders"];

fn a_remote_never_pulled_that_marks_each_contact(sb: &Sandbox) -> [std::path::PathBuf; 2] {
    let resolved = sb.dir.path().join("credential-function-ran");
    sb.add_remote("fake", Remote::at("fake::"));
    sb.write_secrets_lua(&format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = function() \
         io.open({:?}, 'w'):close() return 'tok-123' end }} }} }}\n",
        resolved.display().to_string()
    ));
    [resolved, sb.dir.path().join("token.txt")]
}

fn read_line<'a>(read: &'a str, oid: &'a str, flags: &[&'a str]) -> Vec<&'a str> {
    match read {
        "show" => [&[read, oid][..], flags].concat(),
        _ => [&[read][..], flags].concat(),
    }
}

#[test]
fn a_read_never_starts_a_helper_or_resolves_a_credential() {
    let _guard = support::guard("a_read_never_starts_a_helper_or_resolves_a_credential");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);
    let marks = a_remote_never_pulled_that_marks_each_contact(&sb);

    for read in READS {
        let (ok, _, err) = sb.dam(&read_line(read, &oid, &["-o", "json"]));
        assert!(ok, "{read}: {err}");
        for mark in &marks {
            assert!(
                !mark.exists(),
                "{read} reached the remote: {}",
                mark.display()
            );
        }
    }
}

#[test]
fn no_pull_is_an_unknown_flag_on_every_read() {
    let _guard = support::guard("no_pull_is_an_unknown_flag_on_every_read");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);

    for read in READS {
        let out = sb.output(&read_line(read, &oid, &["--no-pull"]));
        assert_eq!(out.status.code(), Some(2), "{read} took --no-pull");
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(
            said.contains("unexpected argument '--no-pull' found"),
            "{read}: {said}"
        );
    }
}
