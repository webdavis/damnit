use super::*;

#[test]
fn the_first_word_of_each_row_is_an_id_and_blank_lines_are_skipped() {
    for verb in [
        &["done", "-"][..],
        &["edit", "-", "--subject", "x"],
        &["rm", "-"],
    ] {
        assert_eq!(piped_ids(verb, ROWS), ["1111111", "2222222"]);
    }
}

#[test]
fn the_json_document_of_dam_list_gives_the_oids_of_its_objects() {
    let full = "1".repeat(40);
    let document =
        format!(r#"{{"objects": [{{"oid": "{full}", "kind": "task"}}, {{"oid": "abcd"}}]}}"#);
    assert_eq!(
        piped_ids(&["done", "-"], &document),
        [full.as_str(), "abcd"]
    );
}

#[test]
fn mv_takes_its_destination_from_the_path_given_last_or_by_flag() {
    for words in [
        &["mv", "-", "archive/"][..],
        &["mv", "-", "--path", "archive/"],
    ] {
        let cli = piping(words, ROWS).unwrap().unwrap();
        let Command::Mv(args) = &cli.command else {
            panic!()
        };
        assert_eq!(args.path.as_ref().unwrap().as_str(), "archive/");
        assert_eq!(args.set.piped.as_ref().unwrap().ids.len(), 2);
    }
}

#[test]
fn the_piped_set_keeps_the_command_as_typed_for_advice() {
    let cli = piping(&["edit", "-", "--subject", "a b"], ROWS)
        .unwrap()
        .unwrap();
    let (_, set) = set_of(&cli);
    assert_eq!(
        set.piped.as_ref().unwrap().command,
        "dam edit - --subject 'a b'"
    );
}

#[test]
fn ids_on_the_command_line_are_left_as_they_are() {
    let cli = piping(&["done", "1111111"], "").unwrap().unwrap();
    let (oids, set) = set_of(&cli);
    assert_eq!(oids, ["1111111"]);
    assert!(set.piped.is_none());
}

#[test]
fn empty_input_and_blank_lines_alone_change_nothing() {
    assert!(piping(&["done", "-"], "").unwrap().is_none());
    assert!(piping(&["rm", "-"], "\n  \n").unwrap().is_none());
    assert!(
        piping(&["done", "-"], r#"{"objects": []}"#)
            .unwrap()
            .is_none()
    );
}
