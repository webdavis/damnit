use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::CredentialSource;
use crate::secret::Secret;

/// Every credential the helper declared, resolved from the remote's config.
/// A declared name with no spec is `Refusal::MissingCredential`.
pub fn resolve_credentials(
    source: &dyn CredentialSource,
    remote: &RemoteConfig,
    declared: &[String],
) -> Result<Vec<(String, Secret)>, UseCaseError> {
    let mut out = Vec::with_capacity(declared.len());
    for name in declared {
        let spec = remote
            .credentials
            .iter()
            .find(|s| s.name() == name)
            .ok_or_else(|| Refusal::MissingCredential {
                remote: remote.name.0.clone(),
                name: name.clone(),
            })?;
        out.push((name.clone(), source.resolve(spec)?));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CredentialSpec;
    use crate::ports::{CredentialError, RemoteName};

    struct Echo;
    impl CredentialSource for Echo {
        fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
            Ok(match spec {
                CredentialSpec::Literal { value, .. } => value.clone(),
                CredentialSpec::Command { argv, .. } => argv.join(" ").into(),
                CredentialSpec::Env { var, .. } => format!("${var}").into(),
            })
        }
    }

    fn remote(specs: Vec<CredentialSpec>) -> RemoteConfig {
        RemoteConfig {
            name: RemoteName("todoist".into()),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: specs,
            stale: None,
            deadline: None,
            path: None,
        }
    }

    #[test]
    fn each_declared_credential_is_resolved_by_name() {
        let r = remote(vec![CredentialSpec::Literal {
            name: "api_token".into(),
            value: "abc".into(),
        }]);
        let out = resolve_credentials(&Echo, &r, &["api_token".into()]).unwrap();
        assert_eq!(out, vec![("api_token".into(), "abc".into())]);
    }

    #[test]
    fn a_declared_credential_with_no_spec_is_refused_by_name() {
        let r = remote(vec![]);
        let err = resolve_credentials(&Echo, &r, &["api_token".into()]).unwrap_err();
        assert_eq!(
            err,
            UseCaseError::Refused(Refusal::MissingCredential {
                remote: "todoist".into(),
                name: "api_token".into()
            })
        );
    }

    #[test]
    fn nothing_on_the_credential_path_reveals_the_value() {
        const TOKEN: &str = "SUPERSECRETTOKEN";
        let r = remote(vec![CredentialSpec::Literal {
            name: "api_token".into(),
            value: TOKEN.into(),
        }]);
        let resolved = resolve_credentials(&Echo, &r, &["api_token".into()]).unwrap();
        assert!(!format!("{resolved:?}").contains(TOKEN));
        let errors = [
            UseCaseError::from(Refusal::MissingCredential {
                remote: "todoist".into(),
                name: "api_token".into(),
            }),
            UseCaseError::from(CredentialError::Missing("api_token".into())),
            UseCaseError::from(CredentialError::CommandFailed {
                name: "api_token".into(),
                why: "the vault is locked".into(),
            }),
            UseCaseError::from(CredentialError::EnvUnset {
                name: "api_token".into(),
                var: "DAM_TODOIST_API_TOKEN".into(),
            }),
        ];
        for e in errors {
            assert!(!format!("{e}").contains(TOKEN));
            assert!(!format!("{e:?}").contains(TOKEN));
        }
    }

    #[test]
    fn undeclared_specs_are_not_resolved() {
        let r = remote(vec![CredentialSpec::Literal {
            name: "extra".into(),
            value: "x".into(),
        }]);
        assert!(resolve_credentials(&Echo, &r, &[]).unwrap().is_empty());
    }
}
