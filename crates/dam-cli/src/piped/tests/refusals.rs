use super::*;

#[test]
fn a_line_that_does_not_start_with_an_id_is_refused_naming_the_line() {
    let refused = refusal(&["done", "-"], "1111111 a\nno objects match\n");
    assert_eq!(
        refused,
        "line 2 of standard input does not start with an id: \"no objects match\"; \
pipe in the rows of dam list or its -o json document: dam list | dam done -"
    );
}

#[test]
fn json_that_is_not_the_document_of_dam_list_is_refused() {
    assert_eq!(
        refusal(&["rm", "-"], r#"{"oids": ["1111111"]}"#),
        "standard input holds JSON without the objects of dam list -o json; \
pipe that document in: dam list -o json | dam rm -"
    );
}

#[test]
fn a_terminal_as_input_is_refused_with_the_pipe_to_run() {
    let typed = vec!["done".to_string(), "-".to_string()];
    let cli = Cli::try_parse_from(["dam", "done", "-"]).unwrap();
    let refused = with_piped_ids(cli, &mut "".as_bytes(), true, &typed).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "- reads ids from standard input, which is a terminal here; pipe them in: dam list | dam done -"
    );
    assert_eq!(refused.exit_code(), 2);
}

#[test]
fn ids_beside_the_dash_are_refused() {
    assert_eq!(
        refusal(&["done", "-", "1111111"], ROWS),
        "give ids or -, not both: dam list | dam done -"
    );
}

#[test]
fn a_saved_filter_beside_the_dash_is_refused_as_beside_any_id() {
    for line in [
        &["dam", "edit", "-", "--filter", "today"][..],
        &["dam", "edit", "1111111", "--filter", "today"],
    ] {
        assert!(Cli::try_parse_from(line).is_err());
    }
}

#[test]
fn the_dash_beside_a_request_read_from_standard_input_is_refused() {
    let refused = refusal(&["--request", "-", "done", "-"], ROWS);
    assert_eq!(
        refused,
        "standard input is read once, for --request - or for -, not both; \
write the request to a file: dam list | dam --request FILE done -"
    );
}

#[test]
fn the_dash_on_the_line_beside_a_request_read_from_standard_input_is_refused_before_reading() {
    let line = ["dam", "done", "-", "--request", "-"]
        .map(std::ffi::OsString::from)
        .to_vec();
    let Err(refused) = crate::request::parse(line) else {
        panic!("parsed")
    };
    assert_eq!(
        refused.error.to_string(),
        "standard input is read once, for --request - or for -, not both; \
write the request to a file: dam list | dam done - --request FILE"
    );
}
