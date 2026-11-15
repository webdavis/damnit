mod support;

use support::sandbox::Sandbox;

#[test]
fn config_alone_prints_the_help_of_its_commands() {
    let _guard = support::guard("config_alone_prints_the_help_of_its_commands");
    let sb = Sandbox::new();
    let asked = sb.output(&["config", "--help"]);
    for line in [
        &["config"][..],
        &["config", "--json"],
        &["--json", "config"],
    ] {
        let alone = sb.output(line);
        assert_eq!(alone.status.code(), Some(0), "{line:?}: {alone:?}");
        assert!(alone.stderr.is_empty(), "{line:?}: {alone:?}");
        assert_eq!(
            String::from_utf8_lossy(&alone.stdout),
            String::from_utf8_lossy(&asked.stdout),
            "{line:?}"
        );
    }
    assert!(String::from_utf8_lossy(&asked.stdout).contains("print-default"));
}

#[test]
fn config_alone_answers_a_token_and_creates_no_file() {
    let _guard = support::guard("config_alone_answers_a_token_and_creates_no_file");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("fresh").join("dam").join("config.lua");
    let out = sb
        .command(&["config"])
        .env("DAM_CONFIG", &config)
        .env("DAM_TOKEN", "damk_v1_whatever")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("print-default"));
    assert!(!sb.dir.path().join("fresh").exists());
}

#[test]
fn config_alone_into_a_closed_pipe_complains_of_nothing() {
    let _guard = support::guard("config_alone_into_a_closed_pipe_complains_of_nothing");
    let sb = Sandbox::new();
    let (reader, writer) = std::io::pipe().unwrap();
    drop(reader);
    let out = sb.command(&["config"]).stdout(writer).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
}
