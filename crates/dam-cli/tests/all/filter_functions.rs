use crate::support;

use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};
const RAISING: &str = "{ match = function(task, dam)\n\
    error('no such field')\n\
  end }";

#[test]
fn a_raising_filter_names_itself_its_file_and_its_line_to_the_owner() {
    let _guard = support::guard("a_raising_filter_names_itself_its_file_and_its_line_to_the_owner");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    sb.add_filter_written_as("broken", RAISING);
    let out = sb.output(&["list", "--filter", "broken"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = String::from_utf8_lossy(&out.stderr);
    let config = sb.config_path().display().to_string();
    assert!(
        err.starts_with(&format!("dam: filter broken failed at {config}:")),
        "{err}"
    );
    assert!(err.trim_end().ends_with(": no such field"), "{err}");
}

#[test]
fn a_raising_filter_tells_a_token_its_name_and_nothing_from_the_file() {
    let _guard =
        support::guard("a_raising_filter_tells_a_token_its_name_and_nothing_from_the_file");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk", "--due", "2030-01-07"]);
    sb.add_filter_written_as("broken", RAISING);
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "event:read"])]);
    let agenda = [
        "agenda",
        "broken",
        "--from",
        "2030-01-07",
        "--to",
        "2030-01-08",
    ];
    for line in [&["list", "--filter", "broken"][..], &agenda] {
        let out = sb.output_holding(&held, line);
        assert_eq!(out.status.code(), Some(1), "{line:?}: {out:?}");
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "dam: filter broken failed; the owner sees why by running the same command\n"
        );
    }
}

#[test]
fn what_match_prints_goes_to_standard_error_and_leaves_json_whole() {
    let _guard = support::guard("what_match_prints_goes_to_standard_error_and_leaves_json_whole");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    sb.add_filter_written_as(
        "loud",
        "{ match = function(task, dam)\n\
           print('MARK-print') io.write('MARK-write\\n')\n\
           io.stdout:write('MARK-stdout\\n') io.output():write('MARK-output\\n')\n\
           return true\n\
         end }",
    );
    for line in [
        &["list", "--filter", "loud", "-o", "json"][..],
        &["agenda", "loud", "-o", "json"],
    ] {
        let out = sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        serde_json::from_slice::<serde_json::Value>(&out.stdout)
            .unwrap_or_else(|e| panic!("{line:?}: {e}: {out:?}"));
        assert!(
            !String::from_utf8_lossy(&out.stdout).contains("MARK"),
            "{out:?}"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        for way in ["print", "write", "stdout", "output"] {
            assert!(
                stderr.contains(&format!("MARK-{way}\n")),
                "{line:?} {way}: {stderr}"
            );
        }
    }
}
