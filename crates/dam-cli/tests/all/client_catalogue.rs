use crate::support;

use support::sandbox::Sandbox;

#[test]
fn the_catalogue_answers_although_the_store_will_not_open() {
    let _guard = support::guard("the_catalogue_answers_although_the_store_will_not_open");
    let sb = Sandbox::new();
    sb.add_category("effort", &["deep"], true);
    sb.add_filter_written_as("today", "{ due = \"today\" }");

    let a_directory_where_the_store_file_goes = sb.dir.path().join("dam.db");
    std::fs::create_dir(&a_directory_where_the_store_file_goes).unwrap();

    let broken = sb.output(&["list", "-o", "json"]);
    assert_eq!(
        broken.status.code(),
        Some(1),
        "the store opened after all, so this proves nothing: {}",
        String::from_utf8_lossy(&broken.stderr)
    );

    for (args, key, name) in [
        (["category", "list", "-o", "json"], "categories", "effort"),
        (["filter", "list", "-o", "json"], "filters", "today"),
    ] {
        let out = sb.output(&args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?} reached the store: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(answer[key][0]["name"], name, "{answer}");
    }
}
