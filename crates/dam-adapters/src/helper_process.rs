mod conversation;
mod stderr_tail;

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use dam_application::{HelperError, HelperLauncher, RemoteConfig, RemoteHelper, Secret};

use conversation::ProcessHelper;

/// Finds a `dam-remote-<name>` helper executable on a search path and spawns it as a child.
pub struct ProcessLauncher {
    search_path: OsString,
}

impl ProcessLauncher {
    pub fn from_env() -> ProcessLauncher {
        ProcessLauncher {
            search_path: std::env::var_os("PATH").unwrap_or_default(),
        }
    }

    pub fn with_search_path(path: impl Into<OsString>) -> ProcessLauncher {
        ProcessLauncher {
            search_path: path.into(),
        }
    }

    /// The first executable `dam-remote-<helper>` on the search path.
    pub fn find(&self, helper: &str) -> Option<PathBuf> {
        let name = format!("dam-remote-{helper}");
        std::env::split_paths(&self.search_path)
            .map(|dir| dir.join(&name))
            .find(|path| is_executable(path))
    }
}

fn is_executable(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// The environment variable a helper reads one credential from: `DAM_<REMOTE>_<NAME>`,
/// uppercased with every non-alphanumeric character folded to `_`.
pub fn credential_variable(remote: &str, name: &str) -> String {
    let shout = |s: &str| {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>()
    };
    format!("DAM_{}_{}", shout(remote), shout(name))
}

impl HelperLauncher for ProcessLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Ok(Box::new(self.spawn(remote, credentials)?))
    }
}

impl ProcessLauncher {
    fn spawn(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<ProcessHelper, HelperError> {
        let program = self
            .find(&remote.helper)
            .ok_or_else(|| HelperError::NotFound {
                helper: format!("dam-remote-{}", remote.helper),
            })?;
        let mut command = Command::new(program);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (name, value) in credentials {
            command.env(credential_variable(&remote.name.0, name), value.expose());
        }
        let mut child = command
            .spawn()
            .map_err(|e| HelperError::Io(e.to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| HelperError::Io("the child has no stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| HelperError::Io("the child has no stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| HelperError::Io("the child has no stderr".into()))?;
        Ok(ProcessHelper::new(child, stdin, stdout, stderr, remote))
    }
}

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests {
    use dam_application::{HelperError, HelperLauncher};

    use super::testing::{FAKE, install, remote};
    use super::*;

    #[test]
    fn find_locates_the_helper_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), FAKE);
        assert!(launcher.find("t").is_some());
        assert!(launcher.find("missing").is_none());
    }

    #[test]
    fn a_non_executable_helper_is_not_found_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("dam-remote-t");
        std::fs::write(&file, FAKE).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let launcher = ProcessLauncher::with_search_path(dir.path().as_os_str());
        assert!(launcher.find("t").is_none());
        let err = launcher.launch(&remote(), &[]).unwrap_err();
        assert_eq!(
            err,
            HelperError::NotFound {
                helper: "dam-remote-t".into()
            }
        );
    }

    #[test]
    fn a_missing_helper_is_not_found_by_name() {
        let launcher = ProcessLauncher::with_search_path("/nonexistent-dam-dir");
        let err = launcher.launch(&remote(), &[]).unwrap_err();
        assert_eq!(
            err,
            HelperError::NotFound {
                helper: "dam-remote-t".into()
            }
        );
    }

    #[test]
    fn credentials_arrive_as_environment_and_calls_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), FAKE);
        let mut helper = launcher
            .launch(&remote(), &[("api_token".into(), "tok".into())])
            .unwrap();
        let caps = helper.capabilities().unwrap();
        assert_eq!(caps.kinds, vec![dam_domain::Kind::Task]);
        assert_eq!(
            caps.credentials,
            vec!["api_token".to_string(), "tok".to_string()],
            "the credential reached the child as an environment variable"
        );
        assert!(helper.pull(None).unwrap().objects.is_empty());
        let pushed = helper.push(vec![]).unwrap();
        assert_eq!(pushed[0].remote_id.as_deref(), Some("r1"));
    }

    #[test]
    fn an_error_line_is_a_remote_error_and_eof_is_io() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\nread -r line; printf '{\"error\":\"nope\"}\\n'\n",
        );
        let mut helper = launcher.launch(&remote(), &[]).unwrap();
        assert_eq!(
            helper.capabilities().unwrap_err(),
            HelperError::Remote("nope".into())
        );
        assert!(matches!(
            helper.capabilities().unwrap_err(),
            HelperError::Io(_)
        ));
    }

    #[test]
    fn credential_variables_are_upper_snake() {
        assert_eq!(
            credential_variable("todoist", "api_token"),
            "DAM_TODOIST_API_TOKEN"
        );
        assert_eq!(
            credential_variable("my-cal", "client-id"),
            "DAM_MY_CAL_CLIENT_ID"
        );
    }
}
