use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

pub struct Sandbox {
    pub dir: tempfile::TempDir,
}

impl Sandbox {
    pub fn new() -> Sandbox {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let helper = bin.join("dam-remote-fake");
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake_helper.sh"),
            &helper,
        )
        .unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[remote.fake]\nurl = \"fake::\"\ncredentials = [\"api_token\"]\napi_token = \"tok-123\"\n",
        )
        .unwrap();
        Sandbox { dir }
    }

    pub fn command(&self, args: &[&str]) -> Command {
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut command = Command::new(env!("CARGO_BIN_EXE_dam"));
        command
            .args(args)
            .env("PATH", path)
            .env("DAM_CONFIG", self.dir.path().join("config.toml"))
            .env("DAM_STORE", self.dir.path().join("dam.db"))
            .env("FAKE_TOKEN_FILE", self.dir.path().join("token.txt"))
            .env("PUSH_LOG", self.dir.path().join("pushes.jsonl"))
            .env("FAIL_NEXT_PUSH", self.dir.path().join("fail-next-push"))
            .env("HOME", self.dir.path())
            .env("XDG_CONFIG_HOME", self.dir.path().join("config"))
            .env("XDG_DATA_HOME", self.dir.path().join("data"))
            .env("XDG_STATE_HOME", self.dir.path().join("state"))
            .env("TZ", "UTC");
        command
    }

    pub fn spawn(&self, args: &[&str]) -> std::process::Child {
        self.spawn_with_stdin(args, std::process::Stdio::null())
    }

    pub fn spawn_with_stdin(
        &self,
        args: &[&str],
        stdin: std::process::Stdio,
    ) -> std::process::Child {
        self.command(args)
            .stdin(stdin)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    }

    pub fn output(&self, args: &[&str]) -> std::process::Output {
        self.spawn(args).wait_with_output().unwrap()
    }

    pub fn dam(&self, args: &[&str]) -> (bool, String, String) {
        let out = self.spawn(args).wait_with_output().unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    pub fn new_object(&self, args: &[&str]) -> String {
        let mut line = vec!["new"];
        line.extend_from_slice(args);
        let (ok, out, err) = self.dam(&line);
        assert!(ok, "{err}");
        out.split_whitespace().next().unwrap().to_string()
    }

    pub fn fail_next_push(&self) {
        std::fs::write(self.dir.path().join("fail-next-push"), "").unwrap();
    }

    pub fn pushes(&self) -> Vec<serde_json::Value> {
        std::fs::read_to_string(self.dir.path().join("pushes.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    pub fn store_bytes(&self) -> String {
        let db = self.dir.path().join("dam.db");
        let mut bytes = std::fs::read(&db).unwrap();
        bytes.extend(std::fs::read(db.with_extension("db-wal")).unwrap_or_default());
        String::from_utf8_lossy(&bytes).into_owned()
    }

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
