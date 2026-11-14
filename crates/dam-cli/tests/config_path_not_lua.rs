mod support;

use std::path::Path;
use std::process::Output;

use support::sandbox::Sandbox;

fn refused(out: &Output) -> String {
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(stderr.contains("the config is config.lua now"), "{stderr}");
    stderr
}

fn through_the_variable(sb: &Sandbox, config: &Path) -> Output {
    sb.command(&["ls", "--no-pull"])
        .env("DAM_CONFIG", config)
        .output()
        .unwrap()
}

fn through_the_flag(sb: &Sandbox, config: &Path) -> Output {
    let config = config.to_str().unwrap();
    sb.command(&["--config", config, "ls", "--no-pull"])
        .output()
        .unwrap()
}

#[test]
fn an_existing_toml_config_is_refused_without_being_read() {
    let _guard = support::guard("an_existing_toml_config_is_refused_without_being_read");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    std::fs::write(&config, "[remote.fake]\nurl = \"fake::\"\n").unwrap();
    for out in [
        through_the_variable(&sb, &config),
        through_the_flag(&sb, &config),
    ] {
        let said = refused(&out);
        assert!(said.contains(&*config.to_string_lossy()), "{said}");
        assert!(!said.contains("config.toml:"), "{said}");
    }
}

#[test]
fn a_missing_toml_config_is_refused_and_nothing_is_created() {
    let _guard = support::guard("a_missing_toml_config_is_refused_and_nothing_is_created");
    let sb = Sandbox::new();
    let folder = sb.dir.path().join("fresh");
    let config = folder.join("config.toml");
    for out in [
        through_the_variable(&sb, &config),
        through_the_flag(&sb, &config),
    ] {
        refused(&out);
        assert!(!folder.exists(), "dam created {}", folder.display());
    }
}
