use std::collections::BTreeSet;
use std::path::Path;

use super::sandbox::Sandbox;
use super::schema::assert_matches_schema;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Answer {
    pub code: Option<i32>,
    pub out: String,
}

impl Sandbox {
    pub fn doctor(&self, args: &[&str]) -> Answer {
        let out = self.output(args);
        assert!(out.stderr.is_empty(), "{args:?}: {out:?}");
        Answer {
            code: out.status.code(),
            out: String::from_utf8(out.stdout).unwrap(),
        }
    }

    pub fn doctor_json(&self, args: &[&str]) -> (Option<i32>, serde_json::Value) {
        let answer = self.doctor(&[args, &["-o", "json"]].concat());
        let document: serde_json::Value = serde_json::from_str(&answer.out).unwrap();
        assert_matches_schema(&document, "doctor.schema.json");
        (answer.code, document)
    }
}

pub fn report(rows: &[&str], closing: &[&str]) -> String {
    let mut lines = vec![
        format!("dam {VERSION}"),
        String::new(),
        "Setup ── whether this machine is set up to run dam".to_string(),
    ];
    lines.extend(rows.iter().map(|row| row.to_string()));
    lines.extend([String::new(), "Needs attention".to_string()]);
    lines.extend(closing.iter().map(|line| line.to_string()));
    lines.iter().map(|line| format!("{line}\n")).collect()
}

pub const ALL_WELL: &[&str] = &["  ✓ nothing needs attention"];

pub fn listing(folder: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().unwrap().is_dir() {
            names.extend(
                listing(&entry.path())
                    .into_iter()
                    .map(|n| format!("{name}/{n}")),
            );
        }
        names.insert(name);
    }
    names
}
