use crate::support;

use support::sandbox::Sandbox;

fn the_pages_example() -> String {
    let page = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lua.md"),
    )
    .unwrap();
    let (_, rest) = page.split_once("```lua\n").unwrap();
    rest.split_once("```").unwrap().0.to_string()
}

fn run(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn committed(sb: &Sandbox, message: &str) {
    run(sb, &["add", "-A"]);
    run(sb, &["commit", "-m", message]);
}

#[test]
fn the_pages_postponed_example_lists_a_task_whose_due_date_moved_three_times() {
    let _guard =
        support::guard("the_pages_postponed_example_lists_a_task_whose_due_date_moved_three_times");
    let sb = Sandbox::new();
    sb.write_config_lua(&the_pages_example());
    sb.write_secrets_lua("return { schema = 1 }\n");
    let report = sb.new_object(&["write the report", "--due", "2030-01-01"]);
    let invoice = sb.new_object(&["send the invoice", "--due", "2030-01-01"]);
    committed(&sb, "plan");
    for (day, moved) in [
        ("2030-01-02", true),
        ("2030-01-03", false),
        ("2030-01-04", false),
    ] {
        run(&sb, &["edit", &report, "--due", day]);
        if moved {
            run(&sb, &["edit", &invoice, "--due", day]);
        }
        committed(&sb, "postpone");
    }
    let (ok, listed, err) = sb.dam(&["list", "--filter", "postponed"]);
    assert!(ok, "{err}");
    assert!(listed.contains("write the report"), "{listed}");
    assert!(!listed.contains("send the invoice"), "{listed}");
}
