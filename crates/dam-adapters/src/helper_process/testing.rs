use std::os::unix::fs::PermissionsExt;

use dam_application::{RemoteConfig, RemoteName};

use super::ProcessLauncher;

/// A shell helper that answers capabilities with its DAM_T_API_TOKEN value in `kinds`,
/// answers pull with no objects, and echoes push mutations as successes.
pub(super) const FAKE: &str = r#"#!/bin/sh
while IFS= read -r line; do
  case "$line" in
*'"capabilities"'*) printf '{"protocol":1,"kinds":["task","%s"],"fields":["subject"],"credentials":["api_token"],"incremental":false}\n' "$DAM_T_API_TOKEN" ;;
*'"pull"'*) printf '{"objects":[],"removed":[],"sync":null}\n' ;;
*'"push"'*) printf '{"results":[{"oid":"x","ok":true,"remote_id":"r1","why":null}]}\n' ;;
*) printf '{"error":"unknown"}\n' ;;
  esac
done
"#;

pub(super) fn install(dir: &std::path::Path, body: &str) -> ProcessLauncher {
    let file = dir.join("dam-remote-t");
    std::fs::write(&file, body).unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
    ProcessLauncher::with_search_path(dir.as_os_str())
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
