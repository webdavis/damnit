use super::sandbox::Sandbox;

fn hooked() -> Sandbox {
    let sb = Sandbox::new();
    sb.write_hooks(&[
        (
            "task_new",
            "{ run = function() end, on_failure = 'refuse' }",
        ),
        ("task_pulled", "function() end"),
        ("task_done", "'notify'"),
    ]);
    sb
}

fn changing() -> Sandbox {
    let sb = Sandbox::new();
    let (ok, _, err) = sb.dam(&["new", "buy milk"]);
    assert!(ok, "{err}");
    sb.write_hooks(&[(
        "task_new",
        "function() return { subject = 'buy oat milk' } end",
    )]);
    sb
}

fn json_of(sb: &Sandbox, args: &[&str]) -> serde_json::Value {
    let out = sb.output(&[args, &["--json"]].concat());
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

pub fn answer(command: &str) -> Option<Vec<serde_json::Value>> {
    match command {
        "hook list" => Some(vec![json_of(&hooked(), &["hook", "list"])]),
        "hook run" => Some(vec![json_of(&changing(), &["hook", "run", "task_new"])]),
        _ => None,
    }
}
