//! A `dam` run in a temporary home, with a fake helper on `PATH`. Shared by
//! every test binary in this crate that drives the real executable.

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

    /// A `dam` command line in this sandbox, with every path a run could read
    /// pointed inside the temporary directory.
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
            .env("HOME", self.dir.path())
            .env("XDG_CONFIG_HOME", self.dir.path().join("config"))
            .env("XDG_DATA_HOME", self.dir.path().join("data"));
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

    /// The whole outcome of one run, for a case that asserts an exit code.
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

    /// The oid of a newly created object, read off the line `new` printed.
    pub fn new_object(&self, args: &[&str]) -> String {
        let mut line = vec!["new"];
        line.extend_from_slice(args);
        let (ok, out, err) = self.dam(&line);
        assert!(ok, "{err}");
        out.split_whitespace().next().unwrap().to_string()
    }

    /// Replaces the helper with one that reads a request and never answers.
    pub fn install_silent_helper(&self) {
        let helper = self.dir.path().join("bin/dam-remote-fake");
        std::fs::write(&helper, "#!/bin/sh\nwhile :; do sleep 0.05; done\n").unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// True while a helper spawned out of this sandbox's bin directory is running.
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
