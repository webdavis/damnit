use crate::support;

use support::sandbox::Sandbox;

#[test]
fn neither_listing_names_a_staleness_setting() {
    let _guard = support::guard("neither_listing_names_a_staleness_setting");
    let sb = Sandbox::new();

    for line in [
        ["config", "list", "-o", "json"],
        ["remote", "list", "-o", "json"],
        ["config", "list", "-o", "toon"],
        ["remote", "list", "-o", "toon"],
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
