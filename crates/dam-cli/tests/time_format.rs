mod support;

use serde_json::Value;
use support::fixture::{self, CALL_MOM, OAT_MILK, OFFSITE, STANDUP, run, short};
use support::sandbox::Sandbox;

const SMALLEST_EPOCH_SECOND_DAM_COULD_PRINT: i64 = 100_000_000;

fn is_time_key(key: &str) -> bool {
    matches!(
        key,
        "at" | "due" | "deadline" | "start" | "end" | "last_pull" | "last_push"
    ) || key.ends_with("_at")
}

fn is_plain_date(text: &str) -> bool {
    text.len() == 10 && text.parse::<jiff::civil::Date>().is_ok()
}

fn is_rfc_3339(text: &str) -> bool {
    let b = text.as_bytes();
    let n = b.len();
    let names_its_offset =
        text.ends_with('Z') || (n > 6 && matches!(b[n - 6], b'+' | b'-') && b[n - 3] == b':');
    b.get(10) == Some(&b'T') && names_its_offset && text.parse::<jiff::Timestamp>().is_ok()
}

fn starts_like_a_date(text: &str) -> bool {
    text.get(..10)
        .is_some_and(|d| d.parse::<jiff::civil::Date>().is_ok())
}

fn check(command: &str, at: &str, value: &Value, times: &mut usize) {
    match value {
        Value::Object(map) => {
            for (key, v) in map {
                let here = format!("{at}.{key}");
                if is_time_key(key) {
                    assert!(
                        v.is_null() || v.is_string(),
                        "{command}: {here} is a time but prints {v}"
                    );
                }
                check(command, &here, v, times);
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                check(command, &format!("{at}[{i}]"), v, times);
            }
        }
        Value::Number(n) => assert!(
            n.as_i64()
                .is_none_or(|n| n.abs() < SMALLEST_EPOCH_SECOND_DAM_COULD_PRINT),
            "{command}: {at} prints {n}, which reads as epoch seconds"
        ),
        Value::String(s) if starts_like_a_date(s) => {
            assert!(
                is_plain_date(s) || is_rfc_3339(s),
                "{command}: {at} prints {s:?}, neither a plain date nor RFC 3339"
            );
            *times += 1;
        }
        _ => {}
    }
}

fn times_in(sb: &Sandbox, args: &[&str]) -> usize {
    let mut full = vec!["--json"];
    full.extend_from_slice(args);
    let (ok, out, err) = sb.dam(&full);
    assert!(ok, "{args:?}: {err}");
    let document: Value = serde_json::from_str(&out).unwrap();
    let mut times = 0;
    check(&args.join(" "), "", &document, &mut times);
    times
}

fn prints_times(sb: &Sandbox, args: &[&str]) {
    assert!(times_in(sb, args) > 0, "{args:?} printed no time at all");
}

fn shown(sb: &Sandbox, byte: u8, key: &str) -> String {
    let (ok, out, err) = sb.dam(&["--json", "show", &short(byte)]);
    assert!(ok, "{err}");
    let object: Value = serde_json::from_str(&out).unwrap();
    object["event"][key].as_str().unwrap().to_string()
}

#[test]
fn every_command_prints_its_times_as_rfc_3339_or_a_plain_date() {
    let sb = fixture::committed();
    let (oat_milk, call_mom, standup, offsite) = (
        short(OAT_MILK),
        fixture::oid(CALL_MOM),
        short(STANDUP),
        short(OFFSITE),
    );
    let runs: [(&[&str], bool); 23] = [
        (&["ls"], true),
        (&["show", &oat_milk], true),
        (&["show", &standup], true),
        (&["show", &offsite], true),
        (
            &["agenda", "--from", "2026-09-25", "--to", "2026-09-27"],
            true,
        ),
        (&["log"], true),
        (&["filter", "list"], false),
        (&["category", "list"], false),
        (&["done", &oat_milk], false),
        (&["new", "water plants", "--due", "2026-09-26T10:00"], true),
        (
            &[
                "new",
                "retro",
                "--event",
                "--start",
                "2026-09-26T15:00",
                "--end",
                "2026-09-26T16:00",
            ],
            true,
        ),
        (&["mv", &standup, "home/"], true),
        (&["rm", &call_mom], false),
        (&["restore", &call_mom], false),
        (&["status"], true),
        (&["status", "--full"], true),
        (&["diff"], true),
        (&["diff", "--full"], true),
        (&["add", "-A"], true),
        (&["commit", "-m", "plan the week"], true),
        (&["push"], false),
        (&["pull"], false),
        (&["remote", "list"], true),
    ];
    for (args, prints_a_time) in runs {
        let times = times_in(&sb, args);
        assert!(
            !prints_a_time || times > 0,
            "{args:?} printed no time at all"
        );
    }
}

#[test]
fn edit_reads_back_the_times_show_prints() {
    let sb = fixture::committed();
    let standup = short(STANDUP);
    let (start, end) = (shown(&sb, STANDUP, "start"), shown(&sb, STANDUP, "end"));
    run(&sb, &["edit", &standup, "--start", &start, "--end", &end]);
    assert_eq!(shown(&sb, STANDUP, "start"), start);
    assert_eq!(shown(&sb, STANDUP, "end"), end);
}

#[test]
fn a_conflict_prints_both_sides_times_as_rfc_3339_or_a_plain_date() {
    let sb = fixture::committed();
    run(&sb, &["push"]);
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "switch milk"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: Value = serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    run(&sb, &["pull"]);
    prints_times(&sb, &["status"]);
    times_in(&sb, &["resolve", &short(OAT_MILK), "--theirs"]);
}

#[test]
fn token_create_and_list_print_their_terms_as_rfc_3339() {
    let sb = Sandbox::new();
    let value = dam_domain::Token::generate(&mut |b: &mut [u8]| b.fill(7)).expose();
    sb.add_config(&format!(
        "\n[token.later]\nvalue = \"{value}\"\nnot_before = 2998-01-01T00:00:00+02:00\n"
    ));
    prints_times(
        &sb,
        &[
            "token",
            "create",
            "tool",
            "--allow",
            "task:read",
            "--expires",
            "2999-05-11T09:00:00+02:00",
        ],
    );
    assert_eq!(times_in(&sb, &["token", "list"]), 2);
}
