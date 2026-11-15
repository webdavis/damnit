use crate::support;

use support::sandbox::Sandbox;

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
