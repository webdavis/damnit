use crate::support;

use support::sandbox::Sandbox;

#[test]
fn new_add_commit_push_pull_and_ls_work_across_processes() {
    let _guard = support::guard("new_add_commit_push_pull_and_ls_work_across_processes");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&[
        "new",
        "buy oat milk",
        "--due",
        "2026-09-25",
        "-p",
        "1",
        "--label",
        "errand",
    ]);
    assert!(ok, "{err}");
    let oid = out.split_whitespace().next().unwrap().to_string();

    let (ok, out, _) = sb.dam(&["status"]);
    assert!(ok);
    assert!(out.contains("Not staged:"));

    assert!(sb.dam(&["add", "-A"]).0);
    let (ok, out, _) = sb.dam(&["commit", "-m", "triage"]);
    assert!(ok);
    assert!(out.contains("triage"));

    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");
    let token = std::fs::read_to_string(sb.dir.path().join("token.txt")).unwrap();
    assert_eq!(token.trim(), "tok-123");

    let (ok, out, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    assert!(out.contains("✓ fake: 1 new, 1 unchanged"), "{out}");

    let (ok, out, _) = sb.dam(&["ls", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    let subjects: Vec<&str> = json["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap())
        .collect();
    assert!(subjects.contains(&"buy oat milk"));
    assert!(subjects.contains(&"from upstream"));

    let (ok, out, _) = sb.dam(&["ls", "--toon"]);
    assert!(ok);
    assert!(out.starts_with("objects[2]"), "{out}");

    let (ok, out, _) = sb.dam(&["show", &oid[..7]]);
    assert!(ok);
    assert!(out.contains("buy oat milk"));
    assert!(out.contains("p1"), "{out}");
    assert!(out.contains("2026-09-25"), "{out}");
    assert!(out.contains("errand"), "{out}");

    let (ok, _, err) = sb.dam(&["done", &oid[..7]]);
    assert!(ok, "{err}");
    let (ok, out, _) = sb.dam(&["ls", "done", "--json"]);
    assert!(ok);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["objects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn every_recurrence_the_parser_refuses_reports_one_code_and_kind() {
    let _guard = support::guard("every_recurrence_the_parser_refuses_reports_one_code_and_kind");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["probe"]);
    for rule in [
        "every fortnight",
        "every 0 days",
        "every 2147483647 weeks",
        "every month on 40",
        "every day until soon",
    ] {
        let out = sb.output(&["edit", &oid, "--recurrence", rule, "--json"]);
        assert_eq!(out.status.code(), Some(1), "{rule}");
        let said = String::from_utf8_lossy(&out.stderr);
        let document: serde_json::Value = serde_json::from_str(&said)
            .unwrap_or_else(|e| panic!("{rule}: stderr is not one document: {e}\n{said}"));
        assert_eq!(document["error"]["kind"], "parse", "{rule}");
    }
}

#[test]
fn each_exit_code_means_one_thing() {
    let _guard = support::guard("each_exit_code_means_one_thing");
    let sb = Sandbox::new();
    assert_eq!(sb.output(&["ls"]).status.code(), Some(0), "success");

    let a_directory_as_the_store = sb.dir.path().join("wrong");
    std::fs::create_dir(&a_directory_as_the_store).unwrap();
    let broken = sb
        .command(&["ls"])
        .env("DAM_STORE", &a_directory_as_the_store)
        .output()
        .unwrap();
    assert_eq!(broken.status.code(), Some(1), "an internal failure");

    assert_eq!(
        sb.output(&["nonesuch"]).status.code(),
        Some(2),
        "a subcommand dam does not have"
    );
    assert_eq!(
        sb.output(&["new", "x", "-p", "9"]).status.code(),
        Some(2),
        "a flag value dam refuses to read"
    );
    let unreadable = sb.output(&["ls", "due:today due:tomorrow"]);
    assert_eq!(unreadable.status.code(), Some(2), "a query dam cannot read");
    assert_eq!(
        String::from_utf8_lossy(&unreadable.stderr).trim(),
        "dam: expected & or | before due:tomorrow"
    );
}
