use dam_application::Config;

use super::progress_for;
use crate::output::Format;
use crate::progress::{Asked, Shown};

const NOTHING_ASKED: Asked = Asked {
    setting: true,
    progress: false,
    no_progress: false,
    quiet: false,
};

#[test]
fn progress_on_a_terminal_follows_the_progress_setting() {
    let hidden = Config {
        progress: Some(false),
        ..Config::default()
    };
    let shown_on = |config: &Config| progress_for(config, NOTHING_ASKED, Format::Human, true);
    assert_eq!(shown_on(&Config::default()).shown(), Shown::Animated);
    assert_eq!(shown_on(&hidden).shown(), Shown::Hidden);
}

#[test]
fn progress_asked_for_off_a_terminal_writes_records() {
    let asked = Asked {
        progress: true,
        ..NOTHING_ASKED
    };
    let progress = progress_for(&Config::default(), asked, Format::Json, false);
    assert_eq!(progress.shown(), Shown::Records);
}
