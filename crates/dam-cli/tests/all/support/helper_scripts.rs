use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use super::sandbox::Sandbox;

impl Sandbox {
    pub fn install_helper_pulling(&self, task_json: &str) {
        self.install_helper_answering_pull(&format!(
            r#"{{"objects":[{task_json}],"removed":[],"sync":null}}"#
        ));
    }

    pub fn install_helper_answering_pull(&self, answer: &str) {
        let script = format!(
            r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{{"protocol":1,"kinds":["task"],"fields":["subject","due"],"credentials":["api_token"],"incremental":false}}\n' ;;
    *'"pull"'*) printf '%s\n' '{answer}' ;;
    *'"push"'*)
      printf '%s\n' "$line" >> "$PUSH_LOG"
      oid="$(printf '%s' "$line" | grep -o '"oid":"[0-9a-f]*"' | head -1 | cut -d'"' -f4)"
      printf '{{"results":[{{"oid":"%s","ok":true,"remote_id":"up-1","why":null}}]}}\n' "$oid" ;;
    *) printf '{{"error":"unknown request"}}\n' ;;
  esac
done
"#
        );
        let helper = self.dir.path().join("bin/dam-remote-fake");
        std::fs::write(&helper, script).unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

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
