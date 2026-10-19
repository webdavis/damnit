use std::time::Instant;

use super::{EXIT_GRACE_BEFORE_THE_KILL, ProcessHelper, RECHECK_INTERVAL};

impl ProcessHelper {
    pub(super) fn kill_and_reap(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.killed = true;
    }

    fn close_input_so_a_well_behaved_helper_exits(&mut self) {
        drop(self.stdin.take());
    }

    fn ended_or_unwaitable_within_the_exit_grace(&mut self) -> bool {
        let give_up_at = Instant::now() + EXIT_GRACE_BEFORE_THE_KILL;
        while Instant::now() < give_up_at {
            match self.child.try_wait() {
                Ok(None) => std::thread::sleep(RECHECK_INTERVAL),
                Ok(Some(_)) | Err(_) => return true,
            }
        }
        false
    }
}

impl Drop for ProcessHelper {
    fn drop(&mut self) {
        self.close_input_so_a_well_behaved_helper_exits();
        if self.killed || self.ended_or_unwaitable_within_the_exit_grace() {
            return;
        }
        self.kill_and_reap();
    }
}
