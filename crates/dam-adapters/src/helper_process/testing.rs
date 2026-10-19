use std::io::Write;
use std::process::{Command, Stdio};

use dam_application::{RemoteConfig, RemoteName};

use super::ProcessLauncher;
use crate::Interrupt;

pub(super) const FAKE_THAT_ECHOES_ITS_TOKEN: &str = r#"#!/bin/sh
while IFS= read -r line; do
  case "$line" in
*'"capabilities"'*) printf '{"protocol":1,"kinds":["task"],"fields":["subject"],"credentials":["api_token","%s"],"incremental":false}\n' "$DAM_T_API_TOKEN" ;;
*'"pull"'*) printf '{"objects":[],"removed":[],"sync":null}\n' ;;
*'"push"'*) printf '{"results":[{"oid":"%s","ok":true,"remote_id":"r1","why":null}]}\n' "$OID" ;;
*) printf '{"error":"unknown"}\n' ;;
  esac
done
"#;

const VALID_OID_SO_THE_PUSH_ANSWER_TRANSLATES_BACK: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub(super) fn install(dir: &std::path::Path, body: &str) -> ProcessLauncher {
    install_listening_to(dir, body, &Interrupt::default())
}

pub(super) fn install_listening_to(
    dir: &std::path::Path,
    body: &str,
    interrupt: &Interrupt,
) -> ProcessLauncher {
    write_the_executable_from_a_child_so_no_forking_test_inherits_an_open_writer(
        &dir.join("dam-remote-t"),
        &body.replace("$OID", VALID_OID_SO_THE_PUSH_ANSWER_TRANSLATES_BACK),
    );
    ProcessLauncher::with_search_path(dir.as_os_str(), interrupt.clone())
}

fn write_the_executable_from_a_child_so_no_forking_test_inherits_an_open_writer(
    file: &std::path::Path,
    body: &str,
) {
    let mut sh = Command::new("sh")
        .args(["-c", "cat > \"$1\" && chmod 755 \"$1\"", "sh"])
        .arg(file)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    sh.stdin.take().unwrap().write_all(body.as_bytes()).unwrap();
    assert!(
        sh.wait().unwrap().success(),
        "the helper script was not written"
    );
}

pub(super) fn remote() -> RemoteConfig {
    RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        stale: None,
        deadline: None,
        path: None,
    }
}
