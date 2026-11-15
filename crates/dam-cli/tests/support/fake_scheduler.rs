use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Output, Stdio};

use super::sandbox::Sandbox;

impl Sandbox {
    pub fn scheduling(&self, args: &[&str]) -> Output {
        self.scheduling_holding(None, args)
    }

    pub fn scheduling_holding(&self, token: Option<&str>, args: &[&str]) -> Output {
        let bin = self.dir.path().join("bin");
        for program in ["launchctl", "systemctl"] {
            let fake = bin.join(program);
            let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake_scheduler.sh");
            std::fs::copy(script, &fake).unwrap();
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut command = self.command(args);
        command.env("PATH", &bin).stdin(Stdio::null());
        if let Some(token) = token {
            command.env("DAM_TOKEN", token);
        }
        command.output().unwrap()
    }

    pub fn scheduler_calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.dir.path().join("scheduler.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }
}
