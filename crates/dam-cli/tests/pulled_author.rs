mod support;

use serde_json::Value;
use support::hidden::Hidden;
use support::own::{listed, made_by, own_grant};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant_to};

const UPSTREAM: &str = "from upstream";

fn pulled_during_an_own_tokens_read() -> Hidden {
    let sb = Sandbox::new();
    sb.add_config("stale = \"15m\"\n");
    let pull = "[[token.vpt.grant]]\nallow = [\"remote:pull\"]\n";
    let held = grant_to(&sb, "vpt", 1, &format!("{}{pull}", own_grant("")));
    let out = sb.output_holding(&held, &["ls", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let report = owners_listing(&sb, "mine")
        .into_iter()
        .find(|o| o["subject"] == UPSTREAM)
        .expect("the stale remote was pulled")["oid"]
        .as_str()
        .unwrap()
        .to_string();
    Hidden { sb, held, report }
}

fn owners_listing(sb: &Sandbox, by: &str) -> Vec<Value> {
    let out = sb.output(&["ls", "--json", "--no-pull", "--by", by]);
    answer(&out, "ls.schema.json")["objects"]
        .as_array()
        .unwrap()
        .clone()
}

fn owners_text(sb: &Sandbox, by: &str, format: Option<&str>) -> String {
    let mut line = vec!["ls", "--no-pull", "--by", by];
    line.extend(format);
    let (ok, out, err) = sb.dam(&line);
    assert!(ok, "{err}");
    out
}

#[test]
fn a_task_pulled_during_a_tokens_read_is_the_owners() {
    let _guard = support::guard("a_task_pulled_during_a_tokens_read_is_the_owners");
    let h = pulled_during_an_own_tokens_read();
    let mine = owners_listing(&h.sb, "mine");
    assert!(mine.iter().any(|o| o["subject"] == UPSTREAM), "{mine:?}");
    assert!(owners_listing(&h.sb, "agents").is_empty());
    for format in [None, Some("--toon")] {
        assert!(owners_text(&h.sb, "mine", format).contains(UPSTREAM));
        assert!(!owners_text(&h.sb, "agents", format).contains(UPSTREAM));
    }
    let out = h.sb.output(&["show", &h.report, "--json", "--no-pull"]);
    let shown = answer(&out, "show.schema.json");
    assert_eq!(shown["author"], "owner");
    assert!(shown["details"].is_null(), "{shown}");
    let (ok, human, err) = h.sb.dam(&["show", &h.report, "--no-pull"]);
    assert!(ok, "{err}");
    assert!(human.contains("author: owner"), "{human}");
}

#[test]
fn a_task_pulled_during_an_own_tokens_read_answers_it_like_a_missing_one() {
    let _guard =
        support::guard("a_task_pulled_during_an_own_tokens_read_answers_it_like_a_missing_one")
            .allow_slow("three asks, each run hidden and again once removed");
    for ask in [
        ["show", "{}", "--json", "--no-pull"].as_slice(),
        &["edit", "{}", "--subject", "x", "--json"],
        &["done", "{}", "--json"],
    ] {
        let h = pulled_during_an_own_tokens_read();
        assert!(listed(&h, &[]).is_empty());
        let line: Vec<String> = ask.iter().map(|a| a.replace("{}", &h.report)).collect();
        let line: Vec<&str> = line.iter().map(String::as_str).collect();
        let out = h.answers_alike_once_removed(&h.report, &line);
        assert!(!out.status.success(), "{line:?}");
    }
}

#[test]
fn a_task_a_token_made_stays_its_own_once_pushed_and_pulled_back() {
    let _guard = support::guard("a_task_a_token_made_stays_its_own_once_pushed_and_pulled_back");
    let h = pulled_during_an_own_tokens_read();
    let made = made_by(&h.sb, &h.held, "chore", "work");
    let (ok, _, err) = h.sb.dam(&["push"]);
    assert!(ok, "{err}");
    let (ok, _, err) = h.sb.dam(&["pull"]);
    assert!(ok, "{err}");
    let mut agents: Vec<String> = owners_listing(&h.sb, "agents")
        .iter()
        .map(|o| format!("{} by {}", o["subject"], o["author"]))
        .collect();
    agents.sort();
    assert_eq!(agents, ["\"chore\" by \"vpt\""]);
    assert_eq!(listed(&h, &[]), vec![made]);
}
