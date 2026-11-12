use std::io::Write;

use super::fixture::{self, CALL_MOM, OAT_MILK, STANDUP, run, short};
use super::sandbox::Sandbox;

fn json_of(sb: &Sandbox, args: &[&str]) -> serde_json::Value {
    let out = sb.output(&[args, &["--json"]].concat());
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn fed(sb: &Sandbox, args: &[&str], input: &str) -> serde_json::Value {
    let mut child =
        sb.spawn_with_stdin(&[args, &["--json"]].concat(), std::process::Stdio::piped());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn with_token() -> Sandbox {
    let sb = Sandbox::new();
    run(&sb, &["token", "create", "vpt", "--allow", "task:read"]);
    sb
}

fn in_conflict() -> Sandbox {
    let sb = fixture::committed();
    run(&sb, &["push"]);
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "switch milk"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    run(&sb, &["pull"]);
    sb
}

pub fn answers(command: &str) -> Option<Vec<serde_json::Value>> {
    let milk = short(OAT_MILK);
    let every_task_field = [
        "edit",
        &milk,
        "--estimate",
        "30",
        "--focus",
        "today",
        "--attached",
        &short(STANDUP),
        "--depends",
        &short(CALL_MOM),
        "--recurrence",
        "every week",
        "--deadline",
        "2026-09-30",
    ];
    let staged = fixture::with_staged_and_unstaged_changes;
    Some(match command {
        "new" => vec![json_of(
            &Sandbox::new(),
            &["new", "buy oat milk", "--due", "today"],
        )],
        "done" => vec![json_of(&fixture::committed(), &["done", &milk])],
        "edit" => vec![json_of(&fixture::committed(), &every_task_field)],
        "mv" => vec![json_of(&fixture::committed(), &["mv", &milk, "errands/"])],
        "rm" => vec![json_of(&fixture::committed(), &["rm", &short(STANDUP)])],
        "add" => vec![json_of(&fixture::seeded(), &["add", "-A"])],
        "reset" => vec![json_of(&staged(), &["reset"])],
        "restore" => vec![json_of(&staged(), &["restore", &milk])],
        "commit" => {
            let sb = fixture::seeded();
            run(&sb, &["add", "-A"]);
            vec![json_of(&sb, &["commit", "-m", "seed"])]
        }
        "log" => vec![json_of(&fixture::with_two_commits(), &["log"])],
        "show" => {
            let sb = fixture::with_two_commits();
            let commit = json_of(&sb, &["log"])["commits"][0]["id"]
                .as_str()
                .unwrap()
                .to_string();
            vec![
                json_of(&sb, &["show", &milk]),
                json_of(&sb, &["show", &commit]),
            ]
        }
        "status" => vec![json_of(&staged(), &["status", "--full"])],
        "diff" => {
            let sb = staged();
            vec![
                json_of(&sb, &["diff", "--full"]),
                json_of(&sb, &["diff", "--staged", "--full"]),
            ]
        }
        "ls" => {
            let sb = fixture::committed();
            run(&sb, &every_task_field);
            vec![json_of(&sb, &["ls"])]
        }
        "agenda" => vec![json_of(
            &fixture::seeded(),
            &["agenda", "--from", "2026-09-25"],
        )],
        "reminders" => vec![json_of(
            &fixture::seeded(),
            &["reminders", "--from", "2026-09-25"],
        )],
        "day show" => vec![json_of(&Sandbox::new(), &["day", "show", "2026-11-11"])],
        "day set" => vec![fed(
            &Sandbox::new(),
            &["day", "set"],
            r#"{"day":"2026-11-11"}"#,
        )],
        "day list" => vec![json_of(&Sandbox::new(), &["day", "list"])],
        "category list" => vec![json_of(&fixture::seeded(), &["category", "list"])],
        "filter list" => vec![json_of(&fixture::seeded(), &["filter", "list"])],
        "remote add" => vec![json_of(
            &Sandbox::new(),
            &["remote", "add", "cal", "gcal::primary"],
        )],
        "remote list" => vec![json_of(&fixture::pulled_and_pushed(), &["remote", "list"])],
        "push" => vec![json_of(&fixture::committed(), &["push"])],
        "pull" => vec![json_of(&fixture::committed(), &["pull"])],
        "resolve" => vec![json_of(&in_conflict(), &["resolve", &milk, "--theirs"])],
        "token generate" => vec![json_of(&Sandbox::new(), &["token", "generate"])],
        "token create" => vec![json_of(
            &Sandbox::new(),
            &["token", "create", "vpt", "--allow", "task:read"],
        )],
        "token list" => vec![json_of(&with_token(), &["token", "list"])],
        "token revoke" => vec![json_of(&with_token(), &["token", "revoke", "vpt"])],
        _ => return None,
    })
}
