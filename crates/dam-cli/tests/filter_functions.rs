mod support;

use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

const ERRAND_BY_FUNCTION: &str = "{ match = function(task, dam)\n\
    for _, label in ipairs(task.labels) do\n\
      if label == 'errand' then return true end\n\
    end\n\
  end }";

const RAISING: &str = "{ match = function(task, dam)\n\
    error('no such field')\n\
  end }";

fn errands(sb: &Sandbox) {
    sb.new_object(&["buy milk", "--labels", "errand"]);
    sb.new_object(&["post the letter", "--labels", "errand"]);
    sb.new_object(&["write the report"]);
    sb.add_filter("by_text", "@errand");
    sb.add_filter_written_as("by_function", ERRAND_BY_FUNCTION);
}

fn stdout_of(sb: &Sandbox, args: &[&str]) -> String {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{err}");
    out
}

#[test]
fn ls_by_a_function_filter_prints_what_ls_by_a_text_filter_prints() {
    let _guard = support::guard("ls_by_a_function_filter_prints_what_ls_by_a_text_filter_prints");
    let sb = Sandbox::new();
    errands(&sb);
    let by_text = stdout_of(&sb, &["ls", "by_text"]);
    assert_eq!(by_text.lines().count(), 2, "{by_text}");
    assert_eq!(stdout_of(&sb, &["ls", "by_function"]), by_text);
    assert_eq!(
        stdout_of(&sb, &["ls", "by_function", "--json"]),
        stdout_of(&sb, &["ls", "by_text", "--json"])
    );
}

#[test]
fn a_raising_filter_names_itself_its_file_and_its_line_to_the_owner() {
    let _guard = support::guard("a_raising_filter_names_itself_its_file_and_its_line_to_the_owner");
    let sb = Sandbox::new();
    sb.new_object(&["buy milk"]);
    sb.add_filter_written_as("broken", RAISING);
    let out = sb.output(&["ls", "broken"]);
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
    for line in [&["ls", "broken"][..], &agenda] {
        let out = sb.output_holding(&held, line);
        assert_eq!(out.status.code(), Some(1), "{line:?}: {out:?}");
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "dam: filter broken failed; the owner sees why by running the same command\n"
        );
    }
}

#[test]
fn config_validate_accepts_a_function_filter() {
    let _guard = support::guard("config_validate_accepts_a_function_filter");
    let sb = Sandbox::new();
    sb.add_filter_written_as("by_function", ERRAND_BY_FUNCTION);
    let (ok, _, err) = sb.dam(&["config", "validate"]);
    assert!(ok, "{err}");
}

#[test]
fn agenda_by_a_function_filter_keeps_the_events_it_keeps() {
    let _guard = support::guard("agenda_by_a_function_filter_keeps_the_events_it_keeps");
    let sb = Sandbox::new();
    for subject in ["standup", "offsite"] {
        sb.new_object(&[
            "--event",
            subject,
            "--start",
            "2030-01-07T09:00",
            "--end",
            "2030-01-07T09:30",
        ]);
    }
    sb.add_filter_written_as(
        "standups",
        "{ match = function(task, dam) return task.subject == 'standup' end }",
    );
    let window = ["--from", "2030-01-07", "--to", "2030-01-08"];
    let mut args = vec!["agenda", "standups"];
    args.extend(window);
    let agenda = stdout_of(&sb, &args);
    assert!(
        agenda.contains("standup") && !agenda.contains("offsite"),
        "{agenda}"
    );
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
    for line in [&["ls", "loud", "--json"][..], &["agenda", "loud", "--json"]] {
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
