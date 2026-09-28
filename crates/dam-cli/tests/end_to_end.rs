mod support;

use std::os::unix::fs::PermissionsExt;
use std::process::Command;

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
    assert!(out.contains("1 new"), "{out}");

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
fn a_machine_format_refuses_to_open_an_editor_rather_than_running_one() {
    let _guard =
        support::guard("a_machine_format_refuses_to_open_an_editor_rather_than_running_one");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);

    let rewrites_the_template_if_ever_run = sb.dir.path().join("bin/rewrite-template");
    std::fs::write(
        &rewrites_the_template_if_ever_run,
        "#!/bin/sh\nprintf 'subject = \"rewritten by the editor\"\\n' > \"$1\"\n",
    )
    .unwrap();
    std::fs::set_permissions(
        &rewrites_the_template_if_ever_run,
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();

    let refused = sb
        .command(&["edit", "-e", &oid[..7], "--json"])
        .env("EDITOR", &rewrites_the_template_if_ever_run)
        .env("VISUAL", &rewrites_the_template_if_ever_run)
        .output()
        .unwrap();
    assert!(!refused.status.success(), "the editor ran under --json");

    let (ok, out, _) = sb.dam(&["show", &oid[..7], "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        json["subject"].as_str(),
        Some("buy oat milk"),
        "the editor reached the store under --json"
    );
}

#[test]
fn an_interval_no_calendar_holds_is_refused_at_the_edit_so_completion_stays_ordinary() {
    let _guard = support::guard(
        "an_interval_no_calendar_holds_is_refused_at_the_edit_so_completion_stays_ordinary",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["water the plants", "--due", "2026-10-01"]);

    let refused = sb.output(&["edit", &oid, "--recurrence", "every 2147483647 weeks"]);
    assert_eq!(refused.status.code(), Some(1));
    let said = String::from_utf8_lossy(&refused.stderr);
    assert!(said.contains("1 to 1043497 weeks"), "{said}");

    let (ok, out, err) = sb.dam(&["done", &oid]);
    assert!(ok, "{err}");
    assert!(out.contains("done"), "{out}");
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
}

#[test]
fn a_refusal_exits_four_and_names_the_rule() {
    let _guard = support::guard("a_refusal_exits_four_and_names_the_rule");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    assert!(sb.dam(&["new", "child", "--path", "parent/"]).0);
    let output = sb.output(&["done", &parent[..7]]);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&output.stderr).contains("child"));
}

#[test]
fn an_interrupt_during_a_helper_exchange_exits_three_and_kills_the_helper() {
    let _guard =
        support::guard("an_interrupt_during_a_helper_exchange_exits_three_and_kills_the_helper");
    let sb = Sandbox::new();
    sb.install_silent_helper();
    let mut dam = sb.spawn(&["push"]);
    let waited = std::time::Instant::now();
    while !sb.helper_running() {
        assert!(
            waited.elapsed() < std::time::Duration::from_millis(600),
            "the helper never started"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        Command::new("kill")
            .args(["-INT", &dam.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let status = dam.wait().unwrap();
    assert_eq!(status.code(), Some(3), "{status:?}");
    assert!(!sb.helper_running(), "the helper outlived dam");
    let err = stderr_read_once_the_helper_inheriting_it_is_gone(&mut dam);
    assert!(err.trim().ends_with("cancelled"), "{err}");
}

fn stderr_read_once_the_helper_inheriting_it_is_gone(dam: &mut std::process::Child) -> String {
    let mut err = String::new();
    std::io::Read::read_to_string(&mut dam.stderr.take().unwrap(), &mut err).unwrap();
    err
}

fn exit_within_a_second_or_fail_rather_than_hang(
    child: &mut std::process::Child,
) -> std::process::ExitStatus {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while std::time::Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("dam was still running a second after the interrupt");
}

fn drained_on_a_thread_so_a_full_pipe_cannot_deadlock(
    mut pipe: impl std::io::Read + Send + 'static,
) -> std::sync::Arc<std::sync::Mutex<String>> {
    let text = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let sink = std::sync::Arc::clone(&text);
    std::thread::spawn(move || {
        let mut buf = [0u8; 256];
        while let Ok(n) = pipe.read(&mut buf) {
            if n == 0 {
                return;
            }
            sink.lock()
                .unwrap()
                .push_str(&String::from_utf8_lossy(&buf[..n]));
        }
    });
    text
}

#[test]
fn an_interrupt_at_a_prompt_exits_three_instead_of_waiting_for_an_answer() {
    let _guard =
        support::guard("an_interrupt_at_a_prompt_exits_three_instead_of_waiting_for_an_answer");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    assert!(sb.dam(&["new", "child", "--path", "parent/"]).0);
    let mut dam = sb.spawn_with_stdin(
        &["done", &parent[..7], "--force", "--interactive"],
        std::process::Stdio::piped(),
    );
    let _stdin_held_open_so_the_prompt_never_sees_end_of_input = dam.stdin.take().unwrap();
    let asked = drained_on_a_thread_so_a_full_pipe_cannot_deadlock(dam.stderr.take().unwrap());
    let waited = std::time::Instant::now();
    while !asked.lock().unwrap().contains("open child task(s)") {
        assert!(
            waited.elapsed() < std::time::Duration::from_millis(600),
            "the prompt never appeared: {}",
            asked.lock().unwrap()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        Command::new("kill")
            .args(["-INT", &dam.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let status = exit_within_a_second_or_fail_rather_than_hang(&mut dam);
    assert_eq!(status.code(), Some(3), "{status:?}");
    assert!(
        asked.lock().unwrap().trim().ends_with("cancelled"),
        "{}",
        asked.lock().unwrap()
    );
}
