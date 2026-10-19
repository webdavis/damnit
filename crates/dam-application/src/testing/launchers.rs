use std::cell::RefCell;
use std::rc::Rc;

use dam_domain::{Field, Kind};

use crate::ports::HelperError;
use crate::remote::{MutationOutcome, MutationVerdict, PullOutcome, RemoteMutation};
use crate::testing::{ScriptedHelper, ScriptedLauncher, task_caps};

pub type Pushed = Rc<RefCell<Vec<RemoteMutation>>>;

impl ScriptedHelper {
    pub fn answering(pull: Result<PullOutcome, HelperError>) -> ScriptedHelper {
        ScriptedHelper {
            caps: task_caps(),
            pull_answer: pull,
            push_answer: Box::new(|_| Ok(vec![])),
            pushed: Pushed::default(),
            pulled_since: Rc::default(),
        }
    }
}

impl ScriptedLauncher {
    pub fn launching(make: impl Fn() -> ScriptedHelper + 'static) -> ScriptedLauncher {
        ScriptedLauncher {
            make: Box::new(make),
            launched_with: Rc::default(),
        }
    }
}

pub fn answering(pull: Result<PullOutcome, HelperError>) -> ScriptedLauncher {
    ScriptedLauncher::launching(move || ScriptedHelper::answering(pull.clone()))
}

pub fn pulling(outcome: PullOutcome) -> ScriptedLauncher {
    answering(Ok(outcome))
}

pub fn pushing(
    answer: impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static,
) -> (ScriptedLauncher, Pushed) {
    let pushed = Pushed::default();
    let seen = pushed.clone();
    let launcher = ScriptedLauncher::launching(move || {
        let answer = answer.clone();
        ScriptedHelper {
            push_answer: Box::new(move |ms| Ok(answer(ms))),
            pushed: seen.clone(),
            ..ScriptedHelper::answering(Ok(PullOutcome::default()))
        }
    });
    (launcher, pushed)
}

pub fn accepting_all(
    remote_id: Option<&str>,
) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    let remote_id = remote_id.map(str::to_string);
    move |ms| {
        answering_each(
            ms,
            MutationVerdict::Accepted {
                remote_id: remote_id.clone(),
            },
        )
    }
}

pub fn refusing_all(
    why: &str,
) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    let why = why.to_string();
    move |ms| answering_each(ms, MutationVerdict::Refused { why: why.clone() })
}

fn answering_each(ms: &[RemoteMutation], verdict: MutationVerdict) -> Vec<MutationOutcome> {
    ms.iter()
        .map(|m| MutationOutcome {
            oid: m.oid.clone(),
            verdict: verdict.clone(),
        })
        .collect()
}

pub fn pulling_events(fields: Vec<Field>, outcome: PullOutcome) -> ScriptedLauncher {
    ScriptedLauncher::launching(move || {
        let mut helper = ScriptedHelper::answering(Ok(outcome.clone()));
        helper.caps.kinds = vec![Kind::Event];
        helper.caps.fields = fields.clone();
        helper
    })
}
