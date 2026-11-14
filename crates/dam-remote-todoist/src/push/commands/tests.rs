use super::*;

const KEY: &str = "0102030a-0b0c-4d0e-8f10-111213141500";

#[test]
fn a_uuid_numbers_the_command_in_the_character_dam_left_free() {
    assert_eq!(
        repeatable_command_uuid(KEY, 0).unwrap(),
        "0102030a-0b0c-4d0e-8f10-111213141500"
    );
    assert_eq!(
        repeatable_command_uuid(KEY, 2).unwrap(),
        "0102030a-0b0c-4d0e-8f10-111213141502"
    );
    assert_ne!(
        repeatable_command_uuid(KEY, 0).unwrap(),
        repeatable_command_uuid(KEY, 1).unwrap()
    );
}

#[test]
fn a_key_that_is_not_a_uuid_is_refused_rather_than_sliced() {
    for bad in ["", "short", &"x".repeat(36), &"0".repeat(36)] {
        assert!(repeatable_command_uuid(bad, 0).is_err(), "{bad:?}");
    }
    assert!(repeatable_command_uuid(KEY, COMMAND_ORDINALS_IN_ONE_HEX_DIGIT).is_err());
}
