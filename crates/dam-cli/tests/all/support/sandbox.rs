use std::cell::RefCell;
use std::process::Command;

use super::config_files::ConfigFiles;

pub struct Sandbox {
    pub dir: tempfile::TempDir,
    pub(super) files: RefCell<ConfigFiles>,
}

impl Sandbox {
    pub fn new() -> Sandbox {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("bin")).unwrap();
        let sandbox = Sandbox {
            dir,
            files: RefCell::new(ConfigFiles::with_the_fake_remote()),
        };
        sandbox.write_config_files();
        sandbox.install_helper("fake", include_str!("../../fake_helper.sh"));
        sandbox.install_schedulers_that_hold_nothing();
        sandbox
    }

    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_dam"));
        command.args(args);
        self.environ(command)
    }

    pub fn shell(&self, script: &str) -> Command {
        let dam = self.dir.path().join("bin/dam");
        if !dam.exists() {
            std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_dam"), dam).unwrap();
        }
        let mut command = Command::new("bash");
        command.args(["-euo", "pipefail", "-c", script]);
        self.environ(command)
    }

    fn environ(&self, mut command: Command) -> Command {
        let path = format!(
            "{}:{}:{}",
            self.dir.path().join("bin").display(),
            self.dir.path().join("helpers").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        command
            .current_dir(self.dir.path())
            .env("PATH", path)
            .env("DAM_CONFIG", self.config_path())
            .env("DAM_STORE", self.dir.path().join("dam.db"))
            .env("FAKE_TOKEN_FILE", self.dir.path().join("token.txt"))
            .env("PUSH_LOG", self.dir.path().join("pushes.jsonl"))
            .env("FAIL_NEXT_PUSH", self.dir.path().join("fail-next-push"))
            .env(
                "FAIL_FIRST_OF_NEXT_PUSH",
                self.dir.path().join("fail-first-of-next-push"),
            )
            .env("HANG_NEXT_PUSH", self.dir.path().join("hang-next-push"))
            .env(
                "RATE_LIMIT_NEXT_PUSH",
                self.dir.path().join("rate-limit-next-push"),
            )
            .env("HOME", self.dir.path())
            .env_remove("XDG_RUNTIME_DIR")
            .env("XDG_CONFIG_HOME", self.dir.path().join("config"))
            .env("XDG_DATA_HOME", self.dir.path().join("data"))
            .env("XDG_STATE_HOME", self.dir.path().join("state"))
            .env("TZ", "UTC")
            .env("COLUMNS", "80")
            .env_remove("DAM_TOKEN")
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR_FORCE");
        command
    }

    pub fn output_holding(&self, token: &str, args: &[&str]) -> std::process::Output {
        self.command(args)
            .env("DAM_TOKEN", token)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
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
}
