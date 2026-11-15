mod support;

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
fn list_and_get_never_quote_secrets_lua_and_name_the_next_step() {
    let _guard = support::guard("list_and_get_never_quote_secrets_lua_and_name_the_next_step");
    for line in [&["config", "list"][..], &["config", "get", "owner"]] {
        for (secrets, said) in [
            (
                Secrets::Raises,
                "loading stopped at an error; run dam doctor\n",
            ),
            (
                Secrets::Unreadable,
                "Permission denied (os error 13); run dam doctor\n",
            ),
        ] {
            let Some((out, stdout, stderr)) = run(secrets, line) else {
                continue;
            };
            assert_eq!(out.status.code(), Some(1), "{secrets:?} {line:?}");
            assert!(stdout.is_empty(), "{secrets:?} {line:?}");
            assert!(stderr.ends_with(said), "{secrets:?} {line:?}: {stderr}");
            assert!(!stderr.contains("MARKER"), "{stderr}");
        }
    }
}

#[test]
fn list_and_get_answer_without_a_secrets_lua() {
    let _guard = support::guard("list_and_get_answer_without_a_secrets_lua");
    let (out, stdout, stderr) = run(Secrets::Missing, &["config", "list"]).unwrap();
    assert!(out.status.success() && stderr.is_empty(), "{out:?}");
    assert!(stdout.contains("  secrets  no secrets.lua\n"), "{stdout}");
    let (out, stdout, stderr) = run(Secrets::Missing, &["config", "get", "owner"]).unwrap();
    assert!(out.status.success() && stderr.is_empty(), "{out:?}");
    assert_eq!(stdout, "  owner  Ada\n");
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

#[test]
fn validate_quotes_what_secrets_lua_raised_and_creates_nothing() {
    let _guard = support::guard("validate_quotes_what_secrets_lua_raised_and_creates_nothing");
    let line = &["config", "validate"];
    let (out, stdout, stderr) = run(Secrets::Raises, line).unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        stdout.contains("secrets.lua:2: bad token MARKER-s3cret\n"),
        "{stdout}"
    );
    let (out, stdout, stderr) = run(Secrets::Missing, line).unwrap();
    assert!(out.status.success() && stderr.is_empty(), "{out:?}");
    assert!(
        stdout.contains("\n  ✓ config: config.lua is valid, no secrets.lua\n"),
        "{stdout}"
    );
    if let Some((out, stdout, stderr)) = run(Secrets::Unreadable, line) {
        assert_eq!(out.status.code(), Some(1));
        assert!(stderr.is_empty(), "{stderr}");
        assert!(
            stdout.contains("Permission denied (os error 13)\n"),
            "{stdout}"
        );
    }
}
