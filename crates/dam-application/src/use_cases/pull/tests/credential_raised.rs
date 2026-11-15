use dam_domain::Permission::RemotePull;

use crate::access::Access;
use crate::config::CredentialSpec;
use crate::errors::UseCaseError;
use crate::ports::{ComputeFailure, CredentialError, CredentialSource};
use crate::remote::PullOutcome;
use crate::secret::Secret;
use crate::service::{Dam, Remotes};
use crate::testing::{Scenario, allowed, answering, holder, todoist};

struct Raising;

impl CredentialSource for Raising {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        Err(CredentialError::ComputeFailed {
            remote: todoist(),
            name: spec.name().into(),
            at: "secrets.lua:3".into(),
            why: ComputeFailure::Raised {
                message: Some("LEAK_hunter2".into()),
            },
        })
    }
}

fn raised_to(access: &dyn Access) -> UseCaseError {
    let s = Scenario::default();
    let dam = Dam { access, ..s.dam() };
    let launcher = answering(Ok(PullOutcome::default()));
    let remotes = Remotes {
        launcher: &launcher,
        credentials: &Raising,
    };
    dam.pull(remotes, None).unwrap_err()
}

fn failed(message: Option<&str>) -> UseCaseError {
    UseCaseError::Credential(CredentialError::ComputeFailed {
        remote: todoist(),
        name: "api_token".into(),
        at: "secrets.lua:3".into(),
        why: ComputeFailure::Raised {
            message: message.map(str::to_string),
        },
    })
}

#[test]
fn a_token_pulling_is_told_where_the_credential_failed_and_not_its_message() {
    let token = holder(vec![allowed(&[RemotePull], None)]);
    assert_eq!(raised_to(&token), failed(None));
}

#[test]
fn the_owner_pulling_is_told_the_credential_function_s_message() {
    assert_eq!(
        raised_to(&crate::access::Owner),
        failed(Some("LEAK_hunter2"))
    );
}
