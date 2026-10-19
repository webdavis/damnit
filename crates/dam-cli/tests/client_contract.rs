mod support;

use support::errors::error_document;
use support::sandbox::Sandbox;

#[test]
fn a_machine_format_writes_nothing_to_stderr_but_the_document() {
    let _guard = support::guard("a_machine_format_writes_nothing_to_stderr_but_the_document");
    let sb = Sandbox::new();
    std::fs::set_permissions(
        sb.dir.path().join("config.toml"),
        std::os::unix::fs::PermissionsExt::from_mode(0o644),
    )
    .unwrap();

    let added = sb.output(&["remote", "add", "second", "fake::", "--json"]);
    assert_eq!(added.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&added.stderr),
        "",
        "a warning reached the stream a client parses as one document"
    );
    let answer: serde_json::Value =
        serde_json::from_slice(&added.stdout).expect("the answer is one document");
    let warnings = answer["warnings"].as_array().expect("warnings is a list");
    assert_eq!(
        warnings.len(),
        1,
        "the advisory that the config had been readable is in the answer rather than lost: {answer}"
    );
    let said = warnings[0].as_str().unwrap();
    assert!(said.contains("644"), "{said}");
    assert!(!said.contains('/'), "the advisory carries a path: {said}");

    let quiet = sb.output(&["remote", "add", "third", "fake::", "--json"]);
    let answer: serde_json::Value = serde_json::from_slice(&quiet.stdout).unwrap();
    assert_eq!(
        answer["warnings"],
        serde_json::json!([]),
        "an already-private config warns about nothing"
    );

    let failed = sb.output(&["remote", "add", "second", "fake::", "--json"]);
    let err = String::from_utf8_lossy(&failed.stderr);
    serde_json::from_str::<serde_json::Value>(&err)
        .unwrap_or_else(|e| panic!("stderr is not one JSON document: {e}\n{err}"));
}

#[test]
fn a_human_run_keeps_its_plain_line_and_writes_no_document() {
    let _guard = support::guard("a_human_run_keeps_its_plain_line_and_writes_no_document");
    let sb = Sandbox::new();
    let err = String::from_utf8_lossy(&sb.output(&["show", "0000000"]).stderr).into_owned();
    assert!(err.starts_with("dam: "), "{err}");
    assert!(!err.contains('{'), "{err}");
}

#[test]
fn an_editor_is_refused_under_a_machine_format_and_its_death_is_its_own_failure() {
    let _guard = support::guard(
        "an_editor_is_refused_under_a_machine_format_and_its_death_is_its_own_failure",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);
    let dies = sb.dir.path().join("bin/dies");
    std::fs::write(&dies, "#!/bin/sh\nexit 1\n").unwrap();
    std::fs::set_permissions(&dies, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();

    let refused = sb
        .command(&["edit", &oid, "-e", "--json"])
        .env("EDITOR", &dies)
        .env("VISUAL", &dies)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(4));
    let document = error_document(&refused);
    assert_eq!(document["error"]["kind"], "refused");
    assert_eq!(document["error"]["rule"], "needs_an_editor");

    let died = sb
        .command(&["edit", &oid, "-e"])
        .env("EDITOR", &dies)
        .env("VISUAL", &dies)
        .output()
        .unwrap();
    assert_eq!(
        died.status.code(),
        Some(1),
        "an editor that died is dam failing"
    );
    let err = String::from_utf8_lossy(&died.stderr);
    assert!(err.starts_with("dam: editor: "), "{err}");
}
