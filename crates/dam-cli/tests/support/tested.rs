use std::os::unix::fs::PermissionsExt;

use super::doctor::{Answer, VERSION};
use super::sandbox::Sandbox;
use super::schema::assert_matches_schema;

pub const MARKER: &str = "refused-MARKER-5c1e";

pub fn tested(rows: &[&str], closing: &[&str]) -> String {
    let mut lines = vec![
        format!("dam {VERSION}"),
        String::new(),
        "Parts ── whether each part does its real job".to_string(),
    ];
    lines.extend(rows.iter().map(|row| row.to_string()));
    lines.extend([String::new(), "Needs attention".to_string()]);
    lines.extend(closing.iter().map(|line| line.to_string()));
    lines.iter().map(|line| format!("{line}\n")).collect()
}

impl Sandbox {
    pub fn test_json(&self, args: &[&str]) -> (Option<i32>, serde_json::Value) {
        let answer = self.doctor(&[args, &["--json"]].concat());
        let document: serde_json::Value = serde_json::from_str(&answer.out).unwrap();
        assert_matches_schema(&document, "test.schema.json");
        (answer.code, document)
    }

    pub fn test_plain(&self, args: &[&str]) -> Answer {
        self.doctor(args)
    }

    pub fn install_helper_without_login(&self, helper: &str) {
        let log = self.dir.path().join(format!("{helper}-requests"));
        let script = format!(
            "#!/bin/sh\nwhile IFS= read -r line; do\n  printf '%s\\n' \"$line\" >> {log:?}\n  \
             printf '{{\"protocol\":1,\"kinds\":[],\"fields\":[],\"credentials\":[\"api_token\"]}}\\n'\ndone\n"
        );
        let path = self.dir.path().join(format!("bin/dam-remote-{helper}"));
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    pub fn requests_to(&self, helper: &str) -> String {
        let log = self.dir.path().join(format!("{helper}-requests"));
        std::fs::read_to_string(log).unwrap_or_default()
    }
}
