mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use support::sandbox::Sandbox;

const NO_REMOTE_SYNCS: &str = "· no remote has sync_every\n";

fn last_run(sb: &Sandbox) -> PathBuf {
    sb.dir.path().join("state/dam/sync-last-run.txt")
}

fn mode(path: &std::path::Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn with_record_last_run_the_file_holds_only_the_last_runs_report() {
    let _guard = support::guard("with_record_last_run_the_file_holds_only_the_last_run");
    let sb = Sandbox::new();

    let (ok, out, err) = sb.dam(&["sync", "run", "--record-last-run"]);
    assert!(ok, "{err}");
    assert_eq!(
        out, "",
        "the report goes to the file, not to standard output"
    );
    assert_eq!(
        std::fs::read_to_string(last_run(&sb)).unwrap_or_default(),
        NO_REMOTE_SYNCS
    );

    let (ok, out, err) = sb.dam(&["sync", "run", "--record-last-run", "--json"]);
    assert!(ok, "{err}");
    assert_eq!(out, "");
    let (_, printed, _) = sb.dam(&["sync", "run", "--json"]);
    assert_eq!(
        std::fs::read_to_string(last_run(&sb)).unwrap_or_default(),
        printed
    );
    assert_eq!(mode(&last_run(&sb)), 0o600);
    assert_eq!(mode(last_run(&sb).parent().unwrap()), 0o700);
}

#[test]
fn a_run_that_fails_leaves_its_error_in_the_file() {
    let _guard = support::guard("a_run_that_fails_leaves_its_error_in_the_file");
    let sb = Sandbox::new();
    std::fs::write(sb.config_path(), "this is not lua (").unwrap();
    let (_, _, printed) = sb.dam(&["sync", "run"]);

    let out = sb.output(&["sync", "run", "--record-last-run"]);

    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(printed.starts_with("dam: "), "{printed}");
    assert_eq!(
        std::fs::read_to_string(last_run(&sb)).unwrap_or_default(),
        printed
    );
}

#[test]
fn without_record_last_run_no_file_is_made_and_the_report_is_printed() {
    let _guard = support::guard("without_record_last_run_no_file_is_made");
    let sb = Sandbox::new();

    let (ok, out, err) = sb.dam(&["sync", "run"]);

    assert!(ok, "{err}");
    assert_eq!(out, NO_REMOTE_SYNCS);
    assert!(!last_run(&sb).exists());
}

#[test]
fn a_token_is_refused_before_the_file_is_made() {
    let _guard = support::guard("a_token_is_refused_before_the_file_is_made");
    let sb = Sandbox::new();

    let out = sb.output_holding("damk_v1_whatever", &["sync", "run", "--record-last-run"]);

    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(!last_run(&sb).exists());
}

#[test]
fn a_token_is_refused_before_the_file_is_changed() {
    let _guard = support::guard("a_token_is_refused_before_the_file_is_changed");
    let sb = Sandbox::new();
    assert!(sb.dam(&["sync", "run", "--record-last-run"]).0);
    let before = std::fs::read(last_run(&sb)).unwrap();

    let out = sb.output_holding("damk_v1_whatever", &["sync", "run", "--record-last-run"]);

    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_eq!(std::fs::read(last_run(&sb)).unwrap(), before);
}

#[test]
fn empty_first_is_an_unknown_flag() {
    let _guard = support::guard("empty_first_is_an_unknown_flag");
    let sb = Sandbox::new();

    let out = sb.output(&["sync", "run", "--empty-first", "file"]);

    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(!sb.dir.path().join("file").exists());
}
