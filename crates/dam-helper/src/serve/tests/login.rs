use dam_protocol::{Capabilities, Mutation, MutationResult, PullResponse, PushResponse};
use serde_json::{Value, json};

use super::super::serve;
use crate::Remote;

struct SigningIn(Result<(), String>);

impl Remote for SigningIn {
    fn capabilities(&self) -> Capabilities {
        unreachable!("only a login is sent")
    }
    fn pull(&self, _since: Option<&str>) -> Result<PullResponse, String> {
        unreachable!("only a login is sent")
    }
    fn push(
        &self,
        _mutations: Vec<Mutation>,
        _answered: &mut dyn FnMut(&MutationResult),
    ) -> Result<PushResponse, String> {
        unreachable!("only a login is sent")
    }
    fn login(&self) -> Result<(), String> {
        self.0.clone()
    }
}

struct WithoutLogin;

impl Remote for WithoutLogin {
    fn capabilities(&self) -> Capabilities {
        unreachable!("only a login is sent")
    }
    fn pull(&self, _since: Option<&str>) -> Result<PullResponse, String> {
        unreachable!("only a login is sent")
    }
    fn push(
        &self,
        _mutations: Vec<Mutation>,
        _answered: &mut dyn FnMut(&MutationResult),
    ) -> Result<PushResponse, String> {
        unreachable!("only a login is sent")
    }
}

fn answer(remote: &impl Remote) -> Value {
    let mut input = "{\"cmd\":\"login\"}\n".as_bytes();
    let mut out = Vec::new();
    serve(&mut input, &mut out, remote);
    serde_json::from_slice(&out).unwrap()
}

#[test]
fn a_login_that_worked_answers_signed_in() {
    assert_eq!(answer(&SigningIn(Ok(()))), json!({"signed_in": true}));
}

#[test]
fn a_refused_login_answers_what_the_remote_said() {
    let refused = SigningIn(Err("fake answered 401: no".into()));
    assert_eq!(answer(&refused), json!({"error": "fake answered 401: no"}));
}

#[test]
fn a_helper_that_wrote_no_login_says_it_cannot_test_one() {
    assert_eq!(
        answer(&WithoutLogin),
        json!({"error": "this helper cannot test a login"})
    );
}
