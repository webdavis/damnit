mod support;

use support::sandbox::Sandbox;

fn into_a_closed_pipe(sb: &Sandbox, args: &[&str]) -> std::process::Output {
    let (reader, writer) = std::io::pipe().unwrap();
    drop(reader);
    sb.command(args).stdout(writer).output().unwrap()
}

#[test]
fn doctor_into_a_closed_pipe_exits_as_if_read_and_complains_of_nothing() {
    let _guard =
        support::guard("doctor_into_a_closed_pipe_exits_as_if_read_and_complains_of_nothing");
    let sb = Sandbox::new();
    for args in [
        &["doctor"][..],
        &["doctor", "--json"],
        &["doctor", "--toon"],
    ] {
        let out = into_a_closed_pipe(&sb, args);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {out:?}");
        assert!(out.stderr.is_empty(), "{args:?}: {out:?}");
    }
}

#[test]
fn a_listing_into_a_closed_pipe_exits_as_if_read_and_complains_of_nothing() {
    let _guard =
        support::guard("a_listing_into_a_closed_pipe_exits_as_if_read_and_complains_of_nothing");
    let sb = Sandbox::new();
    for args in [&["ls"][..], &["ls", "--json"]] {
        let out = into_a_closed_pipe(&sb, args);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {out:?}");
        assert!(out.stderr.is_empty(), "{args:?}: {out:?}");
    }
}

#[test]
fn a_doctor_report_with_an_issue_into_a_closed_pipe_still_exits_1_and_complains_of_nothing() {
    let _guard = support::guard(
        "a_doctor_report_with_an_issue_into_a_closed_pipe_still_exits_1_and_complains_of_nothing",
    );
    let sb = Sandbox::new();
    sb.write_secrets_lua("return {\n");
    let out = into_a_closed_pipe(&sb, &["doctor", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
}
