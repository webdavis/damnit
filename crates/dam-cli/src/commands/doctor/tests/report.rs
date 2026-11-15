use dam_adapters::SqliteStore;

use super::{FAKE_AND_WORK, FAKE_ONLY, FAKE_TOKEN, Files, Helpers, json, lines_from};
use crate::commands::check_report::CheckReport;

fn listing(files: &Files) -> Vec<(String, u64)> {
    let mut names: Vec<_> = std::fs::read_dir(files.dir.path())
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let size = entry.metadata().unwrap().len();
            (entry.file_name().to_string_lossy().into_owned(), size)
        })
        .collect();
    names.sort();
    names
}

fn report_with_a_store(config_lua: &str) -> (Files, CheckReport) {
    let files = Files::new(config_lua, FAKE_TOKEN);
    drop(SqliteStore::open(&files.store_path()).unwrap());
    let before = listing(&files);
    let report = files.report(&Helpers::default(), &[]);
    assert_eq!(listing(&files), before);
    (files, report)
}

#[test]
fn a_machine_set_up_well_reads_every_check() {
    let (_files, report) = report_with_a_store(FAKE_ONLY);
    let version = format!("  ✓ database: version {}", SqliteStore::SCHEMA_VERSION);
    assert_eq!(
        lines_from(&report, "  ✓ config: both files are valid"),
        [
            "  ✓ config: both files are valid",
            &version,
            "  ✓ helpers: 1 remote: 1 ok",
            "    ✓ fake: dam-remote-fake answered",
            "  ✓ credentials: 1 remote: 1 ok",
            "    ✓ fake: has its credential, not tested",
            "      → run dam test credentials",
            "  ✓ pushes: 1 remote: 1 ok",
            "    ✓ fake: nothing owed",
            "",
            "Needs attention",
            "  ✓ nothing needs attention",
        ]
    );
}

#[test]
fn a_helper_that_did_not_answer_leaves_its_credentials_unchecked() {
    let (_files, report) = report_with_a_store(FAKE_AND_WORK);
    let lines = lines_from(&report, "  ✓ credentials: 2 remotes: 1 ok, 1 not checked");
    assert_eq!(
        lines[..3],
        [
            "  ✓ credentials: 2 remotes: 1 ok, 1 not checked",
            "    ✓ fake: has its credential, not tested",
            "      → run dam test credentials",
        ]
    );
    assert!(
        lines.contains(&"    · work: not checked, its helper did not answer".to_string()),
        "{lines:?}"
    );
    let document = json(&report);
    let results: Vec<(&str, &str)> = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["name"].as_str().unwrap(), c["result"].as_str().unwrap()))
        .collect();
    assert_eq!(
        results,
        [
            ("config", "ok"),
            ("database", "ok"),
            ("helpers", "issue"),
            ("credentials", "ok"),
            ("pushes", "ok"),
        ]
    );
}
