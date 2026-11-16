use super::{Asked, Shown, shown};

fn asked(setting: bool, progress: bool, no_progress: bool, quiet: bool) -> Asked {
    Asked {
        setting,
        progress,
        no_progress,
        quiet,
    }
}

#[test]
fn on_a_terminal_progress_animates_by_default() {
    assert_eq!(
        shown(asked(true, false, false, false), true),
        Shown::Animated
    );
}

#[test]
fn off_a_terminal_nothing_shows_by_default() {
    assert_eq!(
        shown(asked(true, false, false, false), false),
        Shown::Hidden
    );
}

#[test]
fn the_setting_false_hides_it_on_a_terminal() {
    assert_eq!(
        shown(asked(false, false, false, false), true),
        Shown::Hidden
    );
}

#[test]
fn the_progress_flag_animates_on_a_terminal_even_when_the_setting_hides_it() {
    assert_eq!(
        shown(asked(false, true, false, false), true),
        Shown::Animated
    );
}

#[test]
fn the_progress_flag_off_a_terminal_writes_records() {
    assert_eq!(
        shown(asked(true, true, false, false), false),
        Shown::Records
    );
}

#[test]
fn the_no_progress_flag_hides_it_on_a_terminal() {
    assert_eq!(shown(asked(true, false, true, false), true), Shown::Hidden);
}

#[test]
fn quiet_hides_it_on_a_terminal() {
    assert_eq!(shown(asked(true, false, false, true), true), Shown::Hidden);
}
