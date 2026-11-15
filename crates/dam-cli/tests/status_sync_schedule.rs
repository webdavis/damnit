mod support;

use support::config_files::Remote;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const NOT_INSTALLED: &str =
    "warning: sync_every is set on fake and the schedule is not installed; run dam sync enable";

fn warnings(sb: &Sandbox) -> serde_json::Value {
    let status = support::status_json(sb, &["--json"]);
    assert_matches_schema(&status, "status.schema.json");
    status["warnings"].clone()
}

#[test]
fn status_warns_of_sync_every_until_the_schedule_is_installed() {
    let _guard = support::guard("status_warns_of_sync_every_until_the_schedule_is_installed");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().sync_every("15m"));
    assert_eq!(warnings(&sb), serde_json::json!([NOT_INSTALLED]));
    let (_, out, _) = sb.dam(&["status"]);
    assert!(out.starts_with(&format!("{NOT_INSTALLED}\n")), "{out}");

    assert!(sb.scheduling(&["sync", "enable"]).status.success());
    assert_eq!(warnings(&sb), serde_json::json!([]));
}
