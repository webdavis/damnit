use crate::support;

use std::os::unix::fs::PermissionsExt;
use std::process::Output;

use support::sandbox::Sandbox;

const RAISES: &str = "local t = \"MARKER-s3cret\"\nerror(\"bad token \" .. t)\n";

#[derive(Clone, Copy, Debug)]
enum Secrets {
    Raises,
    Missing,
    Unreadable,
}

fn sandbox(secrets: Secrets) -> Option<Sandbox> {
    let sb = Sandbox::new();
    sb.write_config_lua("return { owner = 'Ada' }\n");
    match secrets {
        Secrets::Raises => sb.write_secrets_lua(RAISES),
        Secrets::Missing => std::fs::remove_file(sb.secrets_path()).unwrap(),
        Secrets::Unreadable => {
            sb.write_secrets_lua("return { schema = 1 }\n");
            let mode = std::fs::Permissions::from_mode(0o000);
            std::fs::set_permissions(sb.secrets_path(), mode).unwrap();
            if std::fs::read(sb.secrets_path()).is_ok() {
                eprintln!("skipped: this user reads a file marked unreadable");
                return None;
            }
        }
    }
    Some(sb)
}

fn entries(sb: &Sandbox) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(sb.dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn run(secrets: Secrets, line: &[&str]) -> Option<(Output, String, String)> {
    let sb = sandbox(secrets)?;
    let before = entries(&sb);
    let out = sb.output(line);
    assert_eq!(entries(&sb), before, "{secrets:?} {line:?} created a file");
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    Some((out, stdout, stderr))
}

#[test]
fn print_default_never_reads_secrets_lua() {
    let _guard = support::guard("print_default_never_reads_secrets_lua");
    for secrets in [Secrets::Raises, Secrets::Missing, Secrets::Unreadable] {
        let Some((out, stdout, stderr)) = run(secrets, &["config", "print-default"]) else {
            continue;
        };
        assert!(out.status.success(), "{secrets:?}: {out:?}");
        assert!(stdout.starts_with("-- dam config."), "{secrets:?}");
        assert!(stderr.is_empty(), "{secrets:?}: {stderr}");
    }
}
