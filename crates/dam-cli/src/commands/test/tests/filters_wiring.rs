use dam_adapters::SqliteStore;

use super::{Files, Helpers, rows_of};
use crate::args::PartName;

const SAVED: &str = "return { remote = { fake = { url = \"fake::\" } }, \
                     filter = { soon = { match = function(task) return true end } } }\n";

fn counting_calls(calls: &std::path::Path) -> String {
    format!(
        "return {{ schema = 1, remote = {{ fake = {{ api_token = function()\n  \
         io.open({:?}, 'a'):write('x')\n  return 'tok'\nend }} }} }}\n",
        calls.display().to_string()
    )
}

#[test]
fn filters_run_over_the_store_and_never_run_a_credential_function() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("calls");
    let files = Files::new(SAVED, &counting_calls(&calls));
    SqliteStore::create_if_missing(&files.store_path()).unwrap();
    let report = files.report(&Helpers::default(), &[PartName::Filters]);
    assert_eq!(
        rows_of(&report),
        [
            "  ✓ filters: 1 filter: 1 ok",
            "    ✓ soon: ran on 0 objects, kept 0"
        ]
    );
    assert!(!calls.exists());
}

#[test]
fn filters_with_no_store_yet_are_not_tested() {
    let files = Files::new(SAVED, "return { schema = 1 }\n");
    let report = files.report(&Helpers::default(), &[PartName::Filters]);
    assert_eq!(
        rows_of(&report),
        [
            "  · filters: not tested, there is no store yet",
            "    → run dam status"
        ]
    );
    assert!(!files.store_path().exists());
}
