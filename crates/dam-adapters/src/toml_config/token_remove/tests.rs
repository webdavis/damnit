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
    let before = "[token.tool]\nvalue = \"x\"\n\n[filter.a]\nquery = \"due:today\"\n\n\
                  [[token.tool.grant]]\nallow = [\"task:read\"]\n\n[token.tool.limits]\n";
    assert_eq!(
        removed_from(before, "tool"),
        "\n[filter.a]\nquery = \"due:today\"\n"
    );
}

#[test]
fn a_token_written_inline_or_with_a_quoted_name_goes_too() {
    assert_eq!(
        removed_from(
            "[token]\ntool = { value = \"x\" }\nkeep = { value = \"y\" }\n",
            "tool"
        ),
        "[token]\nkeep = { value = \"y\" }\n"
    );
    assert_eq!(
        removed_from("[token.\"tool\"]\nvalue = \"x\"\n", "tool"),
        ""
    );
}

#[test]
fn a_name_the_config_does_not_hold_leaves_the_file_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "# mine\n").unwrap();
    assert!(remove_token(&path, "tool").is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# mine\n");
}
