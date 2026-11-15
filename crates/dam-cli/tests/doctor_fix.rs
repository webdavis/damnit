mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use support::doctor::listing;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

struct Fresh {
    sb: Sandbox,
    config: PathBuf,
    store: PathBuf,
}

impl Fresh {
    fn new() -> Fresh {
        let sb = Sandbox::new();
        let config = sb.dir.path().join("fresh/conf/config.lua");
        let store = sb.dir.path().join("fresh/data/dam.db");
        Fresh { sb, config, store }
    }

    fn run(&self, args: &[&str]) -> (Option<i32>, String) {
        let out = self
            .sb
            .command(args)
            .env("DAM_CONFIG", &self.config)
            .env("DAM_STORE", &self.store)
            .output()
            .unwrap();
        assert!(out.stderr.is_empty(), "{out:?}");
        (out.status.code(), String::from_utf8(out.stdout).unwrap())
    }

    fn created(&self, args: &[&str]) -> serde_json::Value {
        let (code, out) = self.run(&[args, &["--json"]].concat());
        assert_eq!(code, Some(0), "{out}");
        let document: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_matches_schema(&document, "doctor.schema.json");
        document["created"].clone()
    }
}

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn fix_creates_the_starter_config_and_the_store_and_says_so() {
    let _guard = support::guard("fix_creates_the_starter_config_and_the_store_and_says_so");
    let fresh = Fresh::new();
    let (code, out) = fresh.run(&["doctor", "--fix"]);
    assert_eq!(code, Some(0), "{out}");
    let created = |path: &Path| format!("\n  ✓ created {}\n", path.display());
    assert!(out.contains(&created(&fresh.config)), "{out}");
    assert!(out.contains(&created(&fresh.store)), "{out}");
    assert!(
        out.contains("\n  ✓ config: config.lua is valid, no secrets.lua\n"),
        "{out}"
    );
    let version = format!(
        "\n  ✓ database: version {}\n",
        dam_adapters::SqliteStore::SCHEMA_VERSION
    );
    assert!(out.contains(&version), "{out}");
    let (_, starter) = fresh.run(&["config", "print-default"]);
    assert_eq!(std::fs::read_to_string(&fresh.config).unwrap(), starter);
    assert_eq!(mode(fresh.config.parent().unwrap()), 0o700);
    assert_eq!(mode(fresh.store.parent().unwrap()), 0o700);
    assert_eq!(mode(&fresh.store), 0o600);
}

#[test]
fn the_json_names_each_created_file_and_a_second_fix_creates_nothing() {
    let _guard =
        support::guard("the_json_names_each_created_file_and_a_second_fix_creates_nothing");
    let fresh = Fresh::new();
    assert_eq!(
        fresh.created(&["doctor", "--fix"]),
        serde_json::json!([
            fresh.config.display().to_string(),
            fresh.store.display().to_string()
        ])
    );
    let before = listing(fresh.sb.dir.path());
    assert_eq!(fresh.created(&["doctor", "--fix"]), serde_json::json!([]));
    let (_, out) = fresh.run(&["doctor", "--fix"]);
    assert!(out.contains("\n  · created nothing\n"), "{out}");
    assert_eq!(listing(fresh.sb.dir.path()), before);
}

#[test]
fn without_fix_the_report_has_no_created_key() {
    let _guard = support::guard("without_fix_the_report_has_no_created_key");
    let fresh = Fresh::new();
    let (_, out) = fresh.run(&["doctor", "--json"]);
    let document: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(document.get("created").is_none(), "{document}");
    assert!(!fresh.config.exists() && !fresh.store.exists());
}

#[test]
fn fix_database_creates_only_the_store() {
    let _guard = support::guard("fix_database_creates_only_the_store");
    let fresh = Fresh::new();
    assert_eq!(
        fresh.created(&["doctor", "--fix", "database"]),
        serde_json::json!([fresh.store.display().to_string()])
    );
    assert!(!fresh.config.exists());
}

#[test]
fn fix_config_creates_only_the_config() {
    let _guard = support::guard("fix_config_creates_only_the_config");
    let fresh = Fresh::new();
    assert_eq!(
        fresh.created(&["doctor", "--fix", "config"]),
        serde_json::json!([fresh.config.display().to_string()])
    );
    assert!(!fresh.store.exists());
}

#[test]
fn fixing_a_check_with_nothing_to_fix_creates_nothing() {
    let _guard = support::guard("fixing_a_check_with_nothing_to_fix_creates_nothing");
    let fresh = Fresh::new();
    let before = listing(fresh.sb.dir.path());
    assert_eq!(
        fresh.created(&["doctor", "--fix", "helpers"]),
        serde_json::json!([])
    );
    assert_eq!(listing(fresh.sb.dir.path()), before);
}
