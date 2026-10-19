use super::*;

#[test]
fn an_offset_inside_a_multibyte_character_still_names_a_line() {
    let text = "subject = \"café\"\nbody = \"\"\n";
    let inside = text.find('é').unwrap() + 1;
    assert_eq!(one_based_line_and_column_of_byte(text, inside).0, 1);
    assert_eq!(one_based_line_and_column_of_byte(text, text.len()).0, 3);
}

#[test]
fn a_quoted_key_is_refused_without_a_line_number_rather_than_line_zero() {
    let text = "\"subject\" = 1\n";
    let table = parse_toml(text).unwrap();
    let reader = StrictTable::new(text, &table, Place::Line, &["subject"]).unwrap();
    assert_eq!(
        reader.string("subject").unwrap_err(),
        "subject must be a string"
    );
}

#[test]
fn a_bare_key_is_refused_with_its_line() {
    let text = "body = \"\"\nsubject = 1\n";
    let table = parse_toml(text).unwrap();
    let reader = StrictTable::new(text, &table, Place::Line, &["body", "subject"]).unwrap();
    assert_eq!(
        reader.string("subject").unwrap_err(),
        "line 2: subject must be a string"
    );
}
