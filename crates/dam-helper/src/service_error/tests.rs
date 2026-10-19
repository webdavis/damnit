use super::*;

#[test]
fn every_control_character_becomes_a_space_so_a_notice_cannot_rewrite_the_terminal() {
    assert_eq!(one_line("red\u{1b}[31m\ttext\r\n"), "red [31m text");
}
