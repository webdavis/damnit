mod support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

const RIGHTS: [&str; 5] = [
    "task:create",
    "task:read",
    "task:change",
    "task:complete",
    "task:remove",
];

fn owner(sb: &Sandbox, args: &[&str]) -> Value {
    let mut line = args.to_vec();
    line.push("--json");
    let (ok, out, err) = sb.dam(&line);
    assert!(ok, "{args:?}: {err}");
    serde_json::from_str(&out).unwrap_or(Value::Null)
}

fn snapshot(sb: &Sandbox) -> Vec<Value> {
    ["status", "ls", "diff"]
        .iter()
        .map(|command| owner(sb, &[command]))
        .collect()
}

struct Trip {
    sb: Sandbox,
    held: String,
    trip: String,
    flights: String,
    pack: String,
}

fn full_oid(sb: &Sandbox, new: &[&str]) -> String {
    let short = sb.new_object(new);
    owner(sb, &["show", &short])["oid"]
        .as_str()
        .unwrap()
        .to_string()
}

fn trip_with_a_child_and_a_dependent() -> Trip {
    let sb = Sandbox::new();
    let trip = full_oid(&sb, &["Trip"]);
    let flights = full_oid(&sb, &["Book flights", "--path", "Trip"]);
    let pack = full_oid(&sb, &["Pack"]);
    owner(&sb, &["edit", &pack, "--depends", &trip]);
    owner(&sb, &["add", "-A"]);
    owner(&sb, &["commit", "-m", "seed"]);
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS)]);
    Trip {
        sb,
        held,
        trip,
        flights,
        pack,
    }
}

fn send(t: &Trip, args: &[&str]) -> String {
    let mut line = args.to_vec();
    line.push("--json");
    let out = t.sb.output_holding(&t.held, &line);
    assert!(out.status.success(), "{out:?}");
    let sent: Value = serde_json::from_slice(&out.stdout).unwrap();
    let hash = sent.get("hash").or(sent.get("item")).or(sent.get("oid"));
    hash.unwrap().as_str().unwrap().to_string()
}

fn item_of_kind(t: &Trip, kind: &str) -> String {
    match kind {
        "create" => send(t, &["new", "Renew passport"]),
        "change" => send(t, &["edit", &t.trip, "--subject", "Journey"]),
        "done" => send(t, &["done", &t.flights]),
        _ => send(t, &["rm", &t.trip]),
    }
}

#[test]
fn add_then_reset_of_each_kind_leaves_status_ls_and_diff_as_before() {
    let _guard = support::guard("add_then_reset_of_each_kind_leaves_status_ls_and_diff_as_before");
    for kind in ["create", "change", "done", "removal"] {
        let t = trip_with_a_child_and_a_dependent();
        let hash = item_of_kind(&t, kind);
        let before = snapshot(&t.sb);
        owner(&t.sb, &["add", &hash]);

        owner(&t.sb, &["reset", &hash]);

        let after = snapshot(&t.sb);
        assert_eq!(after, before, "{kind}");
    }
}

#[test]
fn reset_names_a_task_changed_since_the_add_and_leaves_it_as_it_is() {
    let _guard = support::guard("reset_names_a_task_changed_since_the_add_and_leaves_it_as_it_is");
    let t = trip_with_a_child_and_a_dependent();
    let hash = item_of_kind(&t, "removal");
    owner(&t.sb, &["add", &hash]);
    owner(&t.sb, &["edit", &t.pack, "--body", "light"]);

    let out = t.sb.output(&["reset", &hash, "--json"]);

    let said = answer(&out, "reset.schema.json");
    assert_eq!(said["kept"], json!([t.pack]));
    assert_eq!(said["unstaged"], json!([t.trip]));
    let pack = owner(&t.sb, &["show", &t.pack]);
    assert_eq!(pack["body"], "light");
    assert_eq!(pack["depends"], json!([]));
    let flights = owner(&t.sb, &["show", &t.flights]);
    assert_eq!(flights["path"], "Trip/");
}

#[test]
fn reset_names_the_kept_task_in_its_plain_line_and_toon() {
    let _guard = support::guard("reset_names_the_kept_task_in_its_plain_line_and_toon");
    for format in ["plain", "--toon"] {
        let t = trip_with_a_child_and_a_dependent();
        let hash = item_of_kind(&t, "removal");
        owner(&t.sb, &["add", &hash]);
        owner(&t.sb, &["edit", &t.pack, "--body", "light"]);

        let mut line = vec!["reset", hash.as_str()];
        if format != "plain" {
            line.push(format);
        }
        let (ok, out, err) = t.sb.dam(&line);

        assert!(ok, "{err}");
        if format == "plain" {
            let kept = format!("kept {}, changed since dam add", &t.pack[..7]);
            assert!(out.lines().any(|l| l == kept), "{out}");
        } else {
            let said: Value = toon_format::decode_default(&out).unwrap();
            assert_eq!(said["kept"], json!([t.pack]));
        }
    }
}
