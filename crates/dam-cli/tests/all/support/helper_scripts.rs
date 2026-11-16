use std::path::Path;
use std::process::Command;

use super::sandbox::Sandbox;

impl Sandbox {
    pub fn install_helper(&self, helper: &str, script: &str) {
        let name = format!("dam-remote-{helper}");
        let scripts = self.dir.path().join("helper-scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(scripts.join(&name), script).unwrap();
        let launchers = self.dir.path().join("helpers");
        std::fs::create_dir_all(&launchers).unwrap();
        let launcher = launchers.join(&name);
        if !launcher.exists() {
            std::os::unix::fs::symlink(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/helper_launcher.sh"),
                launcher,
            )
            .unwrap();
        }
    }

    pub fn install_silent_helper(&self) {
        self.install_helper("fake", "while :; do sleep 0.05; done\n");
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
            .arg(self.dir.path().join("helper-scripts/dam-remote-fake"))
            .output()
            .unwrap()
            .status
            .success()
    }
}
