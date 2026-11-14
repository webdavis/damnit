use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse, PushResponse, Verdict};
use serde_json::Value;

use super::super::serve;
use crate::Remote;

struct AcceptingEach;

impl Remote for AcceptingEach {
    fn capabilities(&self) -> Capabilities {
        unreachable!("only a push is sent")
    }
    fn pull(&self, _since: Option<&str>) -> Result<PullResponse, String> {
        unreachable!("only a push is sent")
    }
    fn push(
        &self,
        mutations: Vec<Mutation>,
        answered: &mut dyn FnMut(&MutationResult),
    ) -> Result<PushResponse, String> {
        let mut results = Vec::new();
        for m in mutations {
            let result = MutationResult {
                oid: m.oid,
                verdict: Verdict::Accepted {
                    remote_id: Some("r1".into()),
                },
            };
            answered(&result);
            results.push(result);
        }
        Ok(PushResponse {
            results,
            stopped: None,
        })
    }
}

fn lines_answering(request: &str) -> Vec<Value> {
    let mut out = Vec::new();
    serve(
        &mut format!("{request}\n").as_bytes(),
        &mut out,
        &AcceptingEach,
    );
    String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

const TWO_MUTATIONS: &str = r#""mutations":[{"op":"delete","oid":"0101010101010101010101010101010101010101","idempotency_key":"k1","remote_id":"r1"},{"op":"delete","oid":"0202020202020202020202020202020202020202","idempotency_key":"k2","remote_id":"r2"}]"#;

#[test]
fn a_push_that_asks_gets_each_answer_on_its_own_line_before_the_reply() {
    let lines = lines_answering(&format!(
        r#"{{"cmd":"push",{TWO_MUTATIONS},"early_answers":true}}"#
    ));
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert_eq!(lines[0]["early_answer"]["oid"], "01".repeat(20));
    assert_eq!(lines[1]["early_answer"]["oid"], "02".repeat(20));
    assert_eq!(lines[2]["results"].as_array().unwrap().len(), 2);
}

#[test]
fn a_push_that_does_not_ask_gets_one_reply() {
    let lines = lines_answering(&format!(r#"{{"cmd":"push",{TWO_MUTATIONS}}}"#));
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["results"].as_array().unwrap().len(), 2);
}
