use std::os::unix::fs::PermissionsExt;

use dam_application::{RemoteConfig, RemoteName};

use super::ProcessLauncher;

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
    let file = dir.join("dam-remote-t");
    std::fs::write(
        &file,
        body.replace("$OID", VALID_OID_SO_THE_PUSH_ANSWER_TRANSLATES_BACK),
    )
    .unwrap();
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
