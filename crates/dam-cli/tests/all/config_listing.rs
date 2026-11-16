use crate::support;

use support::errors::{error_document, kind_of};
use support::sandbox::Sandbox;
use support::small_config::small_config;

#[test]
fn every_setting_is_listed_with_dams_defaults_marked() {
    let _guard = support::guard("every_setting_is_listed_with_dams_defaults_marked");
    let sb = small_config();
    let (ok, out, err) = sb.dam(&["config", "list"]);
    assert!(ok, "{err}");
    assert!(err.is_empty(), "{err}");
    let expected = format!(
        "\
Files ── where dam reads its configuration
  config   {}
  secrets  {}

Settings ── what dam is using
  owner                                Ada
  done.interactive                     false (default)
  progress                             true (default)
  remote.todoist.url                   todoist::
  remote.todoist.deadline              60s (default)
  remote.todoist.sync.interval         15m (default)
  remote.todoist.sync.allowed          true (default)
  remote.todoist.path                  work/
  remote.todoist.credential.api_token  hidden
  category.effort.values               light, deep
  category.effort.exclusive            true
  filter.today.query                   due:today | overdue
  token.tool.grant                     task:read paths work/
  token.tool.expires                   not set
  token.tool.not_before                not set
  token.tool.via                       local (default)
  token.tool.require                   not set
",
        sb.config_path().display(),
        sb.secrets_path().display(),
    );
    assert_eq!(out, expected);
}

#[test]
fn no_config_lists_the_defaults_and_creates_nothing() {
    let _guard = support::guard("no_config_lists_the_defaults_and_creates_nothing");
    let sb = Sandbox::new();
    let folder = sb.dir.path().join("fresh");
    for format in [&[][..], &["--json"], &["--toon"]] {
        let out = sb
            .command(&[&["config", "list"][..], format].concat())
            .env("DAM_CONFIG", folder.join("config.lua"))
            .output()
            .unwrap();
        assert!(out.status.success(), "{format:?}: {out:?}");
        assert!(out.stderr.is_empty(), "{format:?}: {out:?}");
        assert!(!folder.exists(), "{format:?}");
        if format.is_empty() {
            assert_eq!(
                String::from_utf8(out.stdout).unwrap(),
                "\
Files ── where dam reads its configuration
  config   no config.lua
  secrets  no secrets.lua

Settings ── what dam is using
  owner             owner (default)
  done.interactive  false (default)
  progress          true (default)
"
            );
        }
    }
}

#[test]
fn a_broken_config_lua_gives_validates_error_and_the_next_step() {
    let _guard = support::guard("a_broken_config_lua_gives_validates_error_and_the_next_step");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n  owner = ,\n}\n");
    let listed = sb.output(&["config", "list"]);
    let validated = sb.output(&["config", "validate"]);
    assert_eq!(listed.status.code(), Some(1), "{listed:?}");
    assert!(listed.stdout.is_empty());
    let report = String::from_utf8(validated.stdout).unwrap();
    let reason = report
        .lines()
        .find_map(|line| line.strip_prefix("  ✗ config: "))
        .unwrap();
    let expected = format!("dam: config: {reason}; run dam doctor\n");
    assert_eq!(String::from_utf8(listed.stderr).unwrap(), expected);
    let listed = sb.output(&["config", "list", "--json"]);
    assert_eq!(kind_of(&listed), "parse");
}

#[test]
fn a_token_is_refused_before_the_config_is_read() {
    let _guard = support::guard("a_token_is_refused_before_the_config_is_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let out = sb.output_holding("damk_v1_whatever", &["config", "list", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
}
