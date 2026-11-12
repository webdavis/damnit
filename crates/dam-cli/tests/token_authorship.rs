mod support;

use dam_domain::Token;
use support::sandbox::Sandbox;

fn token(fill: u8) -> Token {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill))
}

#[test]
fn a_pull_a_token_runs_is_recorded_as_the_tokens_change() {
    let _guard = support::guard("a_pull_a_token_runs_is_recorded_as_the_tokens_change");
    let sb = Sandbox::new();
    let held = token(1);
    sb.add_config(&format!(
        "[token.vpt]\nvalue = \"{}\"\n[[token.vpt.grant]]\nallow = [\"remote:pull\"]\n",
        held.expose()
    ));
    let out = sb.output_holding(&held.expose(), &["pull", "fake", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let (ok, log, err) = sb.dam(&["log", "--json"]);
    assert!(ok, "{err}");
    let log: serde_json::Value = serde_json::from_str(&log).unwrap();
    let pulled = &log["commits"][0]["changes"][0];
    assert_eq!(pulled["subject"], "from upstream");
    assert_eq!(pulled["from"], "vpt");
    assert_eq!(pulled["key_id"], held.key_id().as_str());
    assert_eq!(pulled["channel"], "local");
    let (ok, show, _) = sb.dam(&["show", log["commits"][0]["id"].as_str().unwrap()]);
    assert!(ok);
    assert!(show.contains("from upstream  vpt"), "{show}");
}
