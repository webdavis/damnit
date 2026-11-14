use super::*;

#[test]
fn a_remote_name_holds_only_letters_digits_hyphens_and_underscores() {
    assert_eq!(
        RemoteName::parse("to-do_1"),
        Ok(RemoteName("to-do_1".into()))
    );
    assert_eq!(
        RemoteName::parse("a.b").unwrap_err().to_string(),
        "remote name \"a.b\" may hold only letters, digits, - and _"
    );
    for bad in ["bad name", "x]", "a\nb", "caf\u{e9}"] {
        assert_eq!(
            RemoteName::parse(bad),
            Err(RemoteNameError::OutsideTheAlphabet(bad.into()))
        );
    }
    assert_eq!(RemoteName::parse(""), Err(RemoteNameError::Empty));
}
