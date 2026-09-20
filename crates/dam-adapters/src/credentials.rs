use std::process::{Command, Stdio};

use dam_application::{CredentialError, CredentialSource, CredentialSpec, Secret};

type EnvLookup = Box<dyn Fn(&str) -> Option<String>>;

pub struct ProcessCredentialSource {
    env: EnvLookup,
}

impl ProcessCredentialSource {
    pub fn from_env() -> ProcessCredentialSource {
        ProcessCredentialSource::with_env(|k| std::env::var(k).ok())
    }

    pub fn with_env(env: impl Fn(&str) -> Option<String> + 'static) -> ProcessCredentialSource {
        ProcessCredentialSource { env: Box::new(env) }
    }
}

impl CredentialSource for ProcessCredentialSource {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        match spec {
            CredentialSpec::Literal { value, .. } => Ok(value.clone()),
            CredentialSpec::Env { name, var } => {
                (self.env)(var)
                    .map(Secret::from)
                    .ok_or_else(|| CredentialError::EnvUnset {
                        name: name.clone(),
                        var: var.clone(),
                    })
            }
            CredentialSpec::Command { name, argv } => run(name, argv),
        }
    }
}

/// Standard input is inherited so an interactive vault CLI can prompt.
fn run(name: &str, argv: &[String]) -> Result<Secret, CredentialError> {
    let failed = |why: String| CredentialError::CommandFailed {
        name: name.to_string(),
        why,
    };
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| failed("no command given".into()))?;
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| failed(format!("{program}: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(failed(format!(
            "{program} exited {}: {stderr}",
            output.status
        )));
    }
    let value =
        String::from_utf8(output.stdout).map_err(|_| failed("output is not text".into()))?;
    Ok(value.trim_end_matches(['\n', '\r']).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_application::{CredentialError, CredentialSource, CredentialSpec};

    fn no_env() -> ProcessCredentialSource {
        ProcessCredentialSource::with_env(|_| None)
    }

    #[test]
    fn a_literal_is_returned_as_is() {
        let spec = CredentialSpec::Literal {
            name: "api_token".into(),
            value: "v".into(),
        };
        assert_eq!(no_env().resolve(&spec).unwrap().expose(), "v");
    }

    #[test]
    fn a_command_returns_trimmed_stdout() {
        let spec = CredentialSpec::Command {
            name: "api_token".into(),
            argv: vec!["printf".into(), "secret\\n".into()],
        };
        assert_eq!(no_env().resolve(&spec).unwrap().expose(), "secret");
    }

    #[test]
    fn a_failing_command_reports_stderr_and_never_stdout() {
        let spec = CredentialSpec::Command {
            name: "api_token".into(),
            argv: vec![
                "sh".into(),
                "-c".into(),
                "echo leaked; echo locked >&2; exit 3".into(),
            ],
        };
        let err = no_env().resolve(&spec).unwrap_err();
        match err {
            CredentialError::CommandFailed { name, why } => {
                assert_eq!(name, "api_token");
                assert!(why.contains("locked"));
                assert!(!why.contains("leaked"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_missing_command_is_a_command_failure() {
        let spec = CredentialSpec::Command {
            name: "api_token".into(),
            argv: vec!["dam-no-such-binary-zz".into()],
        };
        assert!(matches!(
            no_env().resolve(&spec),
            Err(CredentialError::CommandFailed { .. })
        ));
    }

    #[test]
    fn an_empty_argv_is_a_command_failure_not_a_panic() {
        let spec = CredentialSpec::Command {
            name: "api_token".into(),
            argv: vec![],
        };
        assert!(matches!(
            no_env().resolve(&spec),
            Err(CredentialError::CommandFailed { .. })
        ));
    }

    #[test]
    fn non_utf8_stdout_is_a_command_failure_that_never_echoes_the_bytes() {
        let spec = CredentialSpec::Command {
            name: "api_token".into(),
            argv: vec!["sh".into(), "-c".into(), "printf '\\377'".into()],
        };
        match no_env().resolve(&spec).unwrap_err() {
            CredentialError::CommandFailed { name, why } => {
                assert_eq!(name, "api_token");
                assert!(!why.as_bytes().contains(&0xff));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn env_reads_the_variable_or_reports_it_unset() {
        let source =
            ProcessCredentialSource::with_env(|k| (k == "T").then(|| "from-env".to_string()));
        let set = CredentialSpec::Env {
            name: "api_token".into(),
            var: "T".into(),
        };
        let unset = CredentialSpec::Env {
            name: "api_token".into(),
            var: "U".into(),
        };
        assert_eq!(source.resolve(&set).unwrap().expose(), "from-env");
        assert_eq!(
            source.resolve(&unset).unwrap_err(),
            CredentialError::EnvUnset {
                name: "api_token".into(),
                var: "U".into()
            }
        );
    }
}
