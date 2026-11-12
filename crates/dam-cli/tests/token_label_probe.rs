mod support;

use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

fn with_effort_and_a_work_token() -> (Sandbox, String) {
    let sb = Sandbox::new();
    sb.add_config("[category.effort]\nvalues = [\"light\", \"deep\"]\nexclusive = true\n");
    let held = grant_to(
        &sb,
        "vpt",
        1,
        &format!("{}paths = [\"work\"]\n", grant("\"task:create\"")),
    );
    (sb, held)
}

fn same_answer_whatever_the_labels(sb: &Sandbox, held: &str, line: &[&str]) {
    let answers: Vec<(Option<i32>, Vec<u8>, Vec<u8>)> = [["light", "light"], ["light", "deep"]]
        .into_iter()
        .map(|[first, second]| {
            let mut args = line.to_vec();
            args.extend(["--label", first, "--label", second, "--json"]);
            let out = sb.output_holding(held, &args);
            (out.status.code(), out.stdout, out.stderr)
        })
        .collect();
    assert_eq!(answers[0].0, Some(4), "{:?}", answers[0]);
    assert_eq!(answers[0], answers[1]);
}

#[test]
fn a_new_task_outside_the_grant_answers_the_same_whatever_its_labels() {
    let _guard =
        support::guard("a_new_task_outside_the_grant_answers_the_same_whatever_its_labels");
    let (sb, held) = with_effort_and_a_work_token();
    same_answer_whatever_the_labels(&sb, &held, &["new", "laundry", "--path", "home"]);
}
