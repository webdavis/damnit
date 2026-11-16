use super::*;
use clap::Parser;

#[test]
fn new_takes_a_parent_by_oid_or_prefix() {
    let cli = Cli::try_parse_from(["dam", "new", "buy milk", "--parent", "0101"]).unwrap();
    let Command::New(a) = cli.command else {
        panic!("wrong command")
    };
    assert_eq!(a.parent.as_deref(), Some("0101"));
}

#[test]
fn mv_takes_a_parent_in_place_of_a_path() {
    let cli = Cli::try_parse_from(["dam", "mv", "0303", "--parent", "0101"]).unwrap();
    let Command::Mv(a) = cli.command else {
        panic!("wrong command")
    };
    assert_eq!(a.parent.as_deref(), Some("0101"));
    assert_eq!(a.path, None);
}
