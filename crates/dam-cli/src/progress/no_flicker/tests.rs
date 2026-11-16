use super::leaves_a_line;

#[test]
fn a_step_that_ends_before_its_first_frame_leaves_no_line() {
    assert!(!leaves_a_line(0, false));
    assert!(!leaves_a_line(99, false));
}

#[test]
fn a_step_that_drew_a_frame_or_lived_past_one_leaves_a_line() {
    assert!(leaves_a_line(5, true));
    assert!(leaves_a_line(100, false));
}
