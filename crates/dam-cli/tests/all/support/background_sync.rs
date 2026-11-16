pub fn never_run(spoken: &str) -> serde_json::Value {
    let step = spoken.trim_start_matches("sync ");
    serde_json::json!({
        "remote": "fake",
        "interval_seconds": 900,
        "steps": [{"step": step, "changed": true}]
    })
}
