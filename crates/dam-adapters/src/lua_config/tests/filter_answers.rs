use std::time::{Duration, Instant};

use dam_application::{FilterFailure, FilterFault};

use super::filter_support::{every_field_task, run_filter, run_filter_limited};

fn answered(body: &str) -> Result<Vec<bool>, FilterFailure> {
    run_filter(body, &[every_field_task()])
}

#[test]
fn true_keeps_and_false_and_nil_drop() {
    assert_eq!(answered("        return true").unwrap(), [true]);
    assert_eq!(answered("        return false").unwrap(), [false]);
    assert_eq!(answered("        return nil").unwrap(), [false]);
    assert_eq!(answered("        local quiet = 1").unwrap(), [false]);
}

#[test]
fn a_number_or_a_string_is_refused_naming_what_came_back() {
    for (body, returned) in [("return 1", "an integer"), ("return 'yes'", "a string")] {
        let failure = answered(body).unwrap_err();
        assert_eq!(failure.line, Some(4), "{body}");
        assert_eq!(
            failure.fault,
            FilterFault::Returned {
                returned: returned.into()
            }
        );
    }
}

#[test]
fn a_raised_error_gives_its_line_and_its_message_apart() {
    let failure =
        answered("        local nothing = nil\n        return nothing.field").unwrap_err();
    assert!(failure.file.ends_with("config.lua"), "{:?}", failure.file);
    assert_eq!(failure.line, Some(6));
    let FilterFault::Raised { message } = failure.fault else {
        panic!("{:?}", failure.fault);
    };
    assert!(message.starts_with("attempt to index"), "{message}");
    assert!(!message.contains("config.lua"), "{message}");
}

#[test]
fn a_function_that_never_returns_is_stopped_at_the_limit() {
    let started = Instant::now();
    let failure = run_filter_limited(
        "        while true do end",
        &[every_field_task()],
        Duration::from_millis(200),
    )
    .unwrap_err();
    assert_eq!(failure.fault, FilterFault::RanOver);
    assert_eq!(failure.line, Some(4));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn the_limit_covers_the_whole_listing_not_each_object() {
    let task = every_field_task();
    let many: Vec<_> = (0..200).map(|_| task.clone()).collect();
    let failure = run_filter_limited(
        "        local until_ = os.clock() + 0.05\n        while os.clock() < until_ do end\n        return true",
        &many,
        Duration::from_millis(300),
    )
    .unwrap_err();
    assert_eq!(failure.fault, FilterFault::RanOver);
}
