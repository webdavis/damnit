use std::time::Duration;

use super::fails_the_test;

const PAST_THE_CEILING: Duration = Duration::from_secs(11);

#[test]
fn a_test_past_the_ceiling_passes_when_other_work_shares_the_machine() {
    assert!(!fails_the_test(PAST_THE_CEILING, None, false));
}

#[test]
fn a_test_past_the_ceiling_fails_when_run_alone_on_an_idle_machine() {
    assert!(fails_the_test(PAST_THE_CEILING, None, true));
}

#[test]
fn a_slow_test_with_a_named_cause_passes_even_when_run_alone() {
    assert!(!fails_the_test(PAST_THE_CEILING, Some("a cause"), true));
}

#[test]
fn a_test_under_the_ceiling_passes_when_run_alone() {
    assert!(!fails_the_test(Duration::from_secs(9), None, true));
}
