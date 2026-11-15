mod support;

use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::upstream::{answer_pull_with, task};

const MARKING_THEN_SPINNING: &str = "function(task)\n  \
    io.open(os.getenv('HOME') .. '/hook-started', 'w'):close()\n  \
    local until_ = os.clock() + 0.5\n  \
    while os.clock() < until_ do end\n  \
    return { subject = task.subject .. ' today' }\n\
  end";

fn hooked_with_two_tasks_to_pull() -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!(
        "return {{\n  remote = {{ fake = {{ url = 'fake::' }} }},\n  \
         hook = {{ task_pulled = {MARKING_THEN_SPINNING} }},\n}}\n"
    ));
    answer_pull_with(&sb, json!({"objects": [], "removed": [], "sync": "t1"}));
    assert!(sb.output(&["pull"]).status.success());
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:1", "call Bob", None), task("i:2", "pay rent", None)],
               "removed": [], "sync": "t2"}),
    );
    sb
}

fn json_of(sb: &Sandbox, args: &[&str]) -> Value {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

#[test]
fn ctrl_c_during_the_pulled_hook_exits_three_and_keeps_the_pull_and_its_edit() {
    let _guard =
        support::guard("ctrl_c_during_the_pulled_hook_exits_three_and_keeps_the_pull_and_its_edit")
            .allow_slow("the hook must still be running when the interrupt arrives");
    let sb = hooked_with_two_tasks_to_pull();
    let dam = sb.spawn(&["pull"]);
    let waited = Instant::now();
    while !sb.dir.path().join("hook-started").exists() {
        assert!(
            waited.elapsed() < Duration::from_secs(10),
            "the hook never ran"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let pid = dam.id().to_string();
    assert!(
        Command::new("kill")
            .args(["-INT", &pid])
            .status()
            .unwrap()
            .success()
    );
    let out = dam.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .trim()
            .ends_with("cancelled")
    );
    let listed = json_of(&sb, &["ls", "--json"]);
    let mut subjects: Vec<_> = listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect();
    subjects.sort();
    assert_eq!(subjects, ["call Bob today", "pay rent"]);
    let log = json_of(&sb, &["log", "--json"]);
    assert_eq!(
        log["commits"][0]["message"],
        "task_pulled after pull from fake"
    );
}
