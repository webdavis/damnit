use std::time::Duration;

use dam_application::{Config, ConfigError};
use dam_domain::Path;

use super::support::{Folder, config};

const EXAMPLE: &str = r#"
return {
  owner = "Ada",
  done = { interactive = true },
  remote = {
    todoist = { url = "todoist::", stale = "15m", deadline = "60s", path = "work" },
  },
  category = {
    effort = { values = { "light", "admin", "deep" }, exclusive = true },
  },
  filter = {
    today = { query = "due:today | overdue" },
  },
}
"#;

#[test]
fn config_lua_reads_its_owner_remotes_categories_and_filters() {
    let config = config(EXAMPLE).load().unwrap();
    assert_eq!(config.owner.as_deref(), Some("Ada"));
    assert!(config.done_interactive);
    let remote = config.remote("todoist").unwrap();
    assert_eq!(remote.helper, "todoist");
    assert_eq!(remote.stale, Some(Duration::from_secs(900)));
    assert_eq!(remote.deadline.as_ref().unwrap().text, "60s");
    assert_eq!(remote.path, Some(Path::parse("work").unwrap()));
    assert!(remote.credentials.is_empty());
    let effort = config.categories.category_of_name("effort").unwrap();
    assert_eq!(effort.values, ["light", "admin", "deep"]);
    assert!(effort.exclusive);
    assert_eq!(config.filter("today").unwrap().query, "due:today | overdue");
}

#[test]
fn a_missing_config_lua_reads_as_an_empty_config() {
    assert_eq!(Folder::new().load().unwrap(), Config::default());
}

#[test]
fn a_syntax_error_names_the_file_and_line_and_never_says_toml() {
    let folder = config("return {\n  owner = = 1,\n}\n");
    let ConfigError::StoppedLoading {
        path,
        line: Some(2),
        reason: Some(reason),
    } = folder.refusal()
    else {
        panic!("{:?}", folder.refusal());
    };
    assert_eq!(path, folder.path("config.lua"));
    assert!(!reason.contains("TOML"), "{reason}");
}

#[test]
fn an_error_names_the_whole_path_however_long_the_folder_name() {
    let folder = Folder::under_a_long_folder_name().with_config("return {\n  owner = = 1,\n}\n");
    let ConfigError::StoppedLoading {
        path,
        line: Some(2),
        ..
    } = folder.refusal()
    else {
        panic!("{:?}", folder.refusal());
    };
    assert_eq!(path, folder.path("config.lua"));
}

#[test]
fn a_runtime_error_names_the_file_and_line() {
    let folder = config("local x = nil\nreturn { owner = x.name }\n");
    let ConfigError::StoppedLoading {
        path,
        line: Some(2),
        reason: Some(reason),
    } = folder.refusal()
    else {
        panic!("{:?}", folder.refusal());
    };
    assert_eq!(path, folder.path("config.lua"));
    assert!(!reason.contains("stack traceback"), "{reason}");
}

#[test]
fn a_file_that_returns_no_table_is_refused_by_name() {
    let folder = config("return 42\n");
    assert_eq!(
        folder.refusal(),
        ConfigError::NotATable {
            path: folder.path("config.lua"),
            returned: "an integer".into(),
        }
    );
}
