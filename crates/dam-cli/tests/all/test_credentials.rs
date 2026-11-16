use crate::support;

use support::doctor::listing;
use support::sandbox::Sandbox;

#[test]
fn nothing_is_written_to_the_config_or_store_folders() {
    let _guard = support::guard("nothing_is_written_to_the_config_or_store_folders");
    let sb = Sandbox::new();
    let elsewhere = tempfile::tempdir().unwrap();
    let before = listing(sb.dir.path());
    let out = sb
        .command(&["test"])
        .env("FAKE_TOKEN_FILE", elsewhere.path().join("token.txt"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(listing(sb.dir.path()), before);
}
