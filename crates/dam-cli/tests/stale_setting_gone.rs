mod support;

use support::sandbox::Sandbox;

#[test]
fn a_remote_with_stale_gets_the_unknown_key_answer() {
    let _guard = support::guard("a_remote_with_stale_gets_the_unknown_key_answer");
    let sb = Sandbox::new();
    sb.write_config_lua("return { remote = { fake = { url = 'fake::', stale = '15m' } } }\n");

    let (ok, out, err) = sb.dam(&["ls"]);

    assert!(!ok && out.is_empty(), "{err}");
    assert!(
        err.contains(
            "unknown key stale in remote.fake of config.lua. A remote takes url, deadline, sync_every, path"
        ),
        "{err}"
    );
}

#[test]
fn neither_listing_names_a_staleness_setting() {
    let _guard = support::guard("neither_listing_names_a_staleness_setting");
    let sb = Sandbox::new();

    for line in [
        ["config", "list", "--json"],
        ["remote", "list", "--json"],
        ["config", "list", "--toon"],
        ["remote", "list", "--toon"],
    ] {
        let (ok, out, err) = sb.dam(&line);
        assert!(ok, "{line:?}: {err}");
        assert!(!out.contains("stale"), "{line:?}: {out}");
    }
    for line in [["config", "list"], ["remote", "list"]] {
        let (ok, out, err) = sb.dam(&line);
        assert!(ok, "{line:?}: {err}");
        assert!(!out.contains("stale"), "{line:?}: {out}");
    }
}
