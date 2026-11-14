mod support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

const RIGHTS: [&str; 6] = [
    "task:create",
    "task:read",
    "task:remove",
    "event:create",
    "event:read",
    "event:remove",
];

struct Removal {
    sb: Sandbox,
    held: String,
    target: String,
    other: String,
}

fn owner(sb: &Sandbox, args: &[&str]) -> String {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
    out
}

fn full(sb: &Sandbox, short: &str) -> String {
    let show: Value = serde_json::from_str(&owner(sb, &["show", short, "--json"])).unwrap();
    show["oid"].as_str().unwrap().to_string()
}

fn sent_by_tool_and_tied(sent: &[&str], tie: impl Fn(&Sandbox, &str) -> String) -> Removal {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS).paths(&["work"])]);
    let mut line = sent.to_vec();
    line.extend(["--path", "work", "--json"]);
    let target = answer(&sb.output_holding(&held, &line), "sent.schema.json")["oid"]
        .as_str()
        .unwrap()
        .to_string();
    owner(&sb, &["add", &target]);
    let other = tie(&sb, &target);
    owner(&sb, &["add", "-A"]);
    owner(&sb, &["commit", "-m", "seed"]);
    Removal {
        sb,
        held,
        target,
        other,
    }
}

impl Removal {
    fn sent(&self) -> String {
        let out = self
            .sb
            .output_holding(&self.held, &["rm", &self.target, "--json"]);
        answer(&out, "removal.schema.json")["item"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn waiting_row(&self) -> Value {
        support::status_json(&self.sb, &["--json"])["waiting"][0].clone()
    }

    fn added(&self, item: &str) -> Value {
        let out = self.sb.output(&["add", item, "--json"]);
        answer(&out, "add.schema.json")
    }

    fn staged_by_author(&self) -> Vec<(String, Value, Value)> {
        let status = support::status_json(&self.sb, &["--json"]);
        let mut rows: Vec<(String, Value, Value)> = status["staged"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                let oid = r["oid"].as_str().unwrap().to_string();
                (oid, r["op"].clone(), r["from"].clone())
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }

    fn sorted(&self, mut rows: Vec<(String, Value, Value)>) -> Vec<(String, Value, Value)> {
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }

    fn other_as_owner_sees_it(&self) -> Value {
        serde_json::from_str(&owner(&self.sb, &["show", &self.other, "--json"])).unwrap()
    }
}

#[test]
fn adding_a_removal_moves_its_children_up_as_the_owners_change() {
    let _guard = support::guard("adding_a_removal_moves_its_children_up_as_the_owners_change");
    let r = sent_by_tool_and_tied(&["new", "Trip"], |sb, _| {
        full(sb, &sb.new_object(&["Book flights", "--path", "work/Trip"]))
    });
    let item = r.sent();
    let removes = &r.waiting_row()["removes"];
    assert_eq!(removes["descendants"], json!([r.other]));
    let added = r.added(&item);
    assert_eq!(added["removed"][0]["descendants"], json!([r.other]));
    assert_eq!(r.other_as_owner_sees_it()["path"], "work/");
    assert_eq!(
        r.staged_by_author(),
        r.sorted(vec![
            (r.target.clone(), json!("delete"), json!("tool")),
            (r.other.clone(), json!("update"), Value::Null),
        ])
    );
}

#[test]
fn adding_a_removal_drops_the_dependency_on_it_as_the_owners_change() {
    let _guard = support::guard("adding_a_removal_drops_the_dependency_on_it_as_the_owners_change");
    let r = sent_by_tool_and_tied(&["new", "Draft"], |sb, target| {
        let short = sb.new_object(&["Review", "--path", "home"]);
        owner(sb, &["edit", &short, "--depends", target]);
        full(sb, &short)
    });
    let item = r.sent();
    assert_eq!(r.waiting_row()["removes"]["dependents"], json!([r.other]));
    let held = r.sb.output_holding(&r.held, &["status", "--json"]);
    let held: Value = serde_json::from_slice(&held.stdout).unwrap();
    assert_eq!(held["waiting"][0]["hash"], item.as_str());
    assert_eq!(held["waiting"][0].get("removes"), None);
    let added = r.added(&item);
    assert_eq!(added["removed"][0]["dependents"], json!([r.other]));
    assert_eq!(r.other_as_owner_sees_it()["depends"], json!([]));
    assert_eq!(
        r.staged_by_author(),
        r.sorted(vec![
            (r.target.clone(), json!("delete"), json!("tool")),
            (r.other.clone(), json!("update"), Value::Null),
        ])
    );
    let held = r.sb.output_holding(&r.held, &["status", "--json"]);
    let held = String::from_utf8_lossy(&held.stdout).into_owned();
    assert!(!held.contains(&r.other), "{held}");
}

#[test]
fn adding_an_event_removal_names_the_tasks_still_attached_to_it() {
    let _guard = support::guard("adding_an_event_removal_names_the_tasks_still_attached_to_it");
    let event = [
        "new",
        "Standup",
        "--event",
        "--start",
        "2026-09-25T14:00",
        "--end",
        "2026-09-25T14:30",
    ];
    let r = sent_by_tool_and_tied(&event, |sb, target| {
        let short = sb.new_object(&["Prepare notes", "--path", "home"]);
        owner(sb, &["edit", &short, "--attached", target]);
        full(sb, &short)
    });
    let item = r.sent();
    assert_eq!(r.waiting_row()["removes"]["attached"], json!([r.other]));
    let added = r.added(&item);
    assert_eq!(added["removed"][0]["attached"], json!([r.other]));
    assert_eq!(
        r.staged_by_author(),
        vec![(r.target.clone(), json!("delete"), json!("tool"))]
    );
}
