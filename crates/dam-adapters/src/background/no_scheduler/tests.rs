use dam_application::{BackgroundSync, ScheduleError};

use super::NoScheduler;
use crate::background::testing::job;

#[test]
fn start_and_enable_refuse_with_the_crontab_line_to_add_by_hand() {
    let mut spaced = job();
    spaced.config = "/home/some one/config.lua".into();
    let line = "* * * * * /opt/dam/bin/dam --quiet --config '/home/some one/config.lua' --store /home/someone/dam.db sync run todoist --when-due";
    let expected = Err(ScheduleError::NoScheduler {
        crontab_line: line.into(),
    });
    assert_eq!(NoScheduler.start(&spaced), expected);
    assert_eq!(NoScheduler.enable(&spaced), expected);
}

#[test]
fn stop_and_disable_say_to_remove_the_line_by_hand() {
    for refused in [NoScheduler.stop("todoist"), NoScheduler.disable("todoist")] {
        let Err(ScheduleError::Failed(why)) = refused else {
            panic!("{refused:?}")
        };
        assert!(why.contains("crontab -e"), "{why}");
    }
}
