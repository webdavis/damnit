mod support;

use dam_domain::Token;
use support::sandbox::Sandbox;
use support::token_table::{Grant, TokenTable};

fn token(fill: u8) -> Token {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill))
}

#[test]
fn a_pull_a_token_runs_is_recorded_as_the_owners_change() {
    let _guard = support::guard("a_pull_a_token_runs_is_recorded_as_the_owners_change");
    let sb = Sandbox::new();
    let held = token(1);
    sb.add_token(
        "tool",
        TokenTable::valued(&held.expose()).grant(Grant::allowing(&["remote:pull"])),
    );
    let out = sb.output_holding(&held.expose(), &["pull", "fake", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let (ok, log, err) = sb.dam(&["log", "--json"]);
    assert!(ok, "{err}");
    let log: serde_json::Value = serde_json::from_str(&log).unwrap();
    let pulled = &log["commits"][0]["changes"][0];
    assert_eq!(pulled["subject"], "from upstream");
    for field in ["from", "key_id", "channel"] {
        assert!(pulled[field].is_null(), "{pulled}");
    }
    let (ok, show, _) = sb.dam(&["show", log["commits"][0]["id"].as_str().unwrap()]);
    assert!(ok);
    assert!(!show.contains("tool"), "{show}");
}
