use super::remove_token;

fn removed_from(before: &str, name: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, before).unwrap();
    remove_token(&path, name).unwrap();
    std::fs::read_to_string(&path).unwrap()
}

#[test]
fn every_table_of_the_token_goes_wherever_it_sits() {
    let before = "[token.vpt]\nvalue = \"x\"\n\n[filter.a]\nquery = \"due:today\"\n\n\
                  [[token.vpt.grant]]\nallow = [\"task:read\"]\n\n[token.vpt.limits]\n";
    assert_eq!(
        removed_from(before, "vpt"),
        "\n[filter.a]\nquery = \"due:today\"\n"
    );
}

#[test]
fn a_token_written_inline_or_with_a_quoted_name_goes_too() {
    assert_eq!(
        removed_from(
            "[token]\nvpt = { value = \"x\" }\nkeep = { value = \"y\" }\n",
            "vpt"
        ),
        "[token]\nkeep = { value = \"y\" }\n"
    );
    assert_eq!(removed_from("[token.\"vpt\"]\nvalue = \"x\"\n", "vpt"), "");
}

#[test]
fn a_name_the_config_does_not_hold_leaves_the_file_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "# mine\n").unwrap();
    assert!(remove_token(&path, "vpt").is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# mine\n");
}
