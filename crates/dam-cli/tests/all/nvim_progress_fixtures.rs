use crate::nvim_fixtures::assert_fixture;
use crate::support;

use support::fixture;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn without_seconds(stderr: &[u8]) -> String {
    String::from_utf8(stderr.to_vec())
        .unwrap()
        .lines()
        .map(|line| match line.find("\"seconds\":") {
            Some(at) if line.starts_with("{\"step\"") => {
                let rest = &line[at + "\"seconds\":".len()..];
                let after = rest.find(|c: char| !c.is_ascii_digit()).unwrap();
                format!("{}\"seconds\":0{}", &line[..at], &rest[after..])
            }
            _ => line.to_string(),
        })
        .map(|line| line + "\n")
        .collect()
}

fn progress(sb: &Sandbox, args: &[&str], code: i32) -> String {
    let mut line = args.to_vec();
    line.extend(["--progress", "-o", "json"]);
    let out = sb.output(&line);
    assert_eq!(out.status.code(), Some(code), "{line:?}: {out:?}");
    without_seconds(&out.stderr)
}

fn error_after_records(stderr: &str) -> serde_json::Value {
    let rest: String = stderr
        .lines()
        .filter(|line| !line.starts_with("{\"step\""))
        .collect();
    serde_json::from_str(&rest).unwrap()
}

#[test]
fn the_progress_fixtures_are_what_dam_writes_on_standard_error_for_a_pull_and_a_push() {
    let _guard = support::guard(
        "the_progress_fixtures_are_what_dam_writes_on_standard_error_for_a_pull_and_a_push",
    );
    let sb = fixture::committed();
    assert_fixture("progress-pull.txt", &progress(&sb, &["pull"], 0));
    assert_fixture("progress-push.txt", &progress(&sb, &["push"], 0));
    progress(&sb, &["pull", "fake"], 0);
    progress(&sb, &["push", "fake"], 0);
}

#[test]
fn the_failed_progress_fixture_is_records_then_dams_error_document() {
    let _guard = support::guard("the_failed_progress_fixture_is_records_then_dams_error_document");
    let sb = fixture::committed();
    std::fs::write(sb.dir.path().join("rate-limit-next-push"), "error").unwrap();
    let stderr = progress(&sb, &["push"], 1);
    assert_matches_schema(&error_after_records(&stderr), "error.schema.json");
    assert_fixture("progress-push-failed.txt", &stderr);
}
