use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use super::sandbox::Sandbox;

impl Sandbox {
    pub fn install_silent_helper(&self) {
        let helper = self.dir.path().join("bin/dam-remote-fake");
        std::fs::write(&helper, "#!/bin/sh\nwhile :; do sleep 0.05; done\n").unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    pub fn no_helper_left_within(&self, margin: std::time::Duration) -> bool {
        let give_up_at = std::time::Instant::now() + margin;
        while self.helper_running() {
            if std::time::Instant::now() >= give_up_at {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        true
    }

    pub fn helper_running(&self) -> bool {
        Command::new("pgrep")
            .arg("-f")
            .arg(self.dir.path().join("bin/dam-remote-fake"))
            .output()
            .unwrap()
            .status
            .success()
    }
}
