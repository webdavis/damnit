mod support;

use support::sandbox::Sandbox;

#[test]
fn show_given_a_prefix_too_short_to_be_an_oid_says_so_and_exits_2() {
    let _guard = support::guard("show_given_a_prefix_too_short_to_be_an_oid_says_so_and_exits_2");
    let sb = Sandbox::new();
    sb.new_object(&["buy oat milk"]);
    let short = sb.output(&["show", "--no-pull", "f5"]);
    assert_eq!(short.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&short.stderr),
        "dam: \"f5\" is not an oid; give at least 4 hex characters\n"
    );
}
