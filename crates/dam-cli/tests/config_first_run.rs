mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use support::sandbox::Sandbox;

fn fresh_config(sb: &Sandbox) -> PathBuf {
    sb.dir.path().join("fresh").join("dam").join("config.toml")
}

fn run(sb: &Sandbox, config: &Path, args: &[&str]) -> Output {
    sb.command(args).env("DAM_CONFIG", config).output().unwrap()
}

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn the_first_run_creates_the_config_readable_by_its_owner_alone() {
    let _guard = support::guard("the_first_run_creates_the_config_readable_by_its_owner_alone");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    let out = run(&sb, &config, &["ls"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(mode(&config), 0o600);
    assert_eq!(mode(config.parent().unwrap()), 0o700);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(
        stderr.matches(&*config.to_string_lossy()).count(),
        1,
        "{stderr}"
    );
    let text = std::fs::read_to_string(&config).unwrap();
    for section in [
        "[done]",
        "[remote.",
        "[category.",
        "[filter.",
        "schema",
        "[token.",
    ] {
        assert!(
            text.contains(section),
            "{section} is not explained:\n{text}"
        );
    }
}

#[test]
fn the_created_config_is_read_without_complaint() {
    let _guard = support::guard("the_created_config_is_read_without_complaint");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    run(&sb, &config, &["ls"]);
    let out = run(&sb, &config, &["status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(answer["warnings"], serde_json::json!([]));
}

#[test]
fn a_machine_reader_gets_clean_streams_on_the_first_run() {
    let _guard = support::guard("a_machine_reader_gets_clean_streams_on_the_first_run");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    for flag in ["--json", "--toon"] {
        let out = run(&sb, &config, &["filter", "list", flag]);
        assert!(out.status.success(), "{out:?}");
        assert!(out.stderr.is_empty(), "{out:?}");
        assert!(String::from_utf8(out.stdout).unwrap().contains("filters"));
    }
    assert!(config.exists());
}

#[test]
fn an_existing_config_is_never_rewritten() {
    let _guard = support::guard("an_existing_config_is_never_rewritten");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    run(&sb, &config, &["ls"]);
    std::fs::write(&config, "[filter.mine]\nquery = \"p1\"\n").unwrap();
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o640)).unwrap();
    let out = run(&sb, &config, &["ls"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        "[filter.mine]\nquery = \"p1\"\n"
    );
    assert_eq!(mode(&config), 0o640);
}

#[test]
fn runs_that_start_together_create_one_whole_config() {
    let _guard = support::guard("runs_that_start_together_create_one_whole_config");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    let children: Vec<_> = (0..8)
        .map(|_| {
            sb.command(&["filter", "list"])
                .env("DAM_CONFIG", &config)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let notices: usize = children
        .into_iter()
        .map(|child| {
            let out = child.wait_with_output().unwrap();
            assert!(out.status.success(), "{out:?}");
            String::from_utf8(out.stderr).unwrap().lines().count()
        })
        .sum();
    assert_eq!(notices, 1);
    let ours = std::fs::read_to_string(&config).unwrap();
    let sb2 = Sandbox::new();
    let theirs = fresh_config(&sb2);
    run(&sb2, &theirs, &["filter", "list"]);
    assert_eq!(ours, std::fs::read_to_string(&theirs).unwrap());
    let leftovers = std::fs::read_dir(config.parent().unwrap()).unwrap().count();
    assert_eq!(leftovers, 1);
}

#[test]
fn a_token_caller_creates_nothing() {
    let _guard = support::guard("a_token_caller_creates_nothing");
    let sb = Sandbox::new();
    let config = fresh_config(&sb);
    let out = sb
        .command(&["ls", "--json"])
        .env("DAM_CONFIG", &config)
        .env("DAM_TOKEN", "damk_v1_whatever")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(5), "{out:?}");
    assert!(!config.exists());
    assert!(!config.parent().unwrap().exists());
}
