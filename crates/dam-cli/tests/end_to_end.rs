//! Drives the real `dam` binary through a fake helper on `PATH`, proving the
//! whole promise: new/add/commit/push reaches a helper with the credential,
//! pull brings an upstream object in, ls renders both formats, and a second
//! process sees the same store.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new() -> Sandbox {
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
            "[remote.fake]\nurl = \"fake::\"\napi_token = \"tok-123\"\n",
        )
        .unwrap();
        Sandbox { dir }
    }

    fn dam(&self, args: &[&str]) -> (bool, String, String) {
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let out = Command::new(env!("CARGO_BIN_EXE_dam"))
            .args(args)
            .env("PATH", path)
            .env("DAM_CONFIG", self.dir.path().join("config.toml"))
            .env("DAM_STORE", self.dir.path().join("dam.db"))
            .env("FAKE_TOKEN_FILE", self.dir.path().join("token.txt"))
            .env("HOME", self.dir.path())
            .output()
            .unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }
}

#[test]
fn new_add_commit_push_pull_and_ls_work_across_processes() {
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&[
        "new",
        "buy oat milk",
        "--due",
        "2026-09-25",
        "-p",
        "1",
        "--label",
        "errand",
    ]);
    assert!(ok, "{err}");
    let oid = out.split_whitespace().next().unwrap().to_string();

    let (ok, out, _) = sb.dam(&["status"]);
    assert!(ok);
    assert!(out.contains("Not staged:"));

    assert!(sb.dam(&["add", "-A"]).0);
    let (ok, out, _) = sb.dam(&["commit", "-m", "triage"]);
    assert!(ok);
    assert!(out.contains("triage"));

    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");
    let token = std::fs::read_to_string(sb.dir.path().join("token.txt")).unwrap();
    assert_eq!(token.trim(), "tok-123");

    let (ok, out, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    assert!(out.contains("1 new"), "{out}");

    let (ok, out, _) = sb.dam(&["ls", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    let subjects: Vec<&str> = json["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap())
        .collect();
    assert!(subjects.contains(&"buy oat milk"));
    assert!(subjects.contains(&"from upstream"));

    let (ok, out, _) = sb.dam(&["ls", "--toon"]);
    assert!(ok);
    assert!(out.starts_with("objects[2]"), "{out}");

    let (ok, out, _) = sb.dam(&["show", &oid[..7]]);
    assert!(ok);
    assert!(out.contains("buy oat milk"));

    let (ok, _, err) = sb.dam(&["done", &oid[..7]]);
    assert!(ok, "{err}");
    let (ok, out, _) = sb.dam(&["ls", "done", "--json"]);
    assert!(ok);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["objects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_refusal_exits_two_and_names_the_rule() {
    let sb = Sandbox::new();
    let (_, out, _) = sb.dam(&["new", "parent"]);
    let parent = out.split_whitespace().next().unwrap().to_string();
    assert!(sb.dam(&["new", "child", "--path", "parent/"]).0);
    let output = Command::new(env!("CARGO_BIN_EXE_dam"))
        .args(["done", &parent[..7]])
        .env("DAM_CONFIG", sb.dir.path().join("config.toml"))
        .env("DAM_STORE", sb.dir.path().join("dam.db"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("child"));
}
