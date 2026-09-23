mod conversation;
mod stderr_tail;

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use dam_application::{HelperError, HelperLauncher, RemoteConfig, RemoteHelper, Secret};
use dam_protocol::credential_variable;

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
        // Git's invocation: the remote's name, then the address its url names.
        command.arg(&remote.name.0).arg(remote.address());
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Every non-alphanumeric character folds to an underscore, so two
        // names of one remote can land on one variable. Refuse rather than
        // hand the helper whichever of the two was written last.
        let mut taken: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();
        for (name, value) in credentials {
            let variable = credential_variable(&remote.name.0, name);
            if let Some(first) = taken.insert(variable.clone(), name) {
                return Err(HelperError::CollidingCredentials {
                    first: first.to_string(),
                    second: name.clone(),
                    variable,
                });
            }
            command.env(variable, value.expose());
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
    use dam_application::{HelperError, HelperLauncher, RemoteName};

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
    fn two_credential_names_that_become_one_variable_are_refused_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), FAKE);
        let err = launcher
            .launch(
                &remote(),
                &[
                    ("client-id".into(), "one".into()),
                    ("client.id".into(), "two".into()),
                ],
            )
            .unwrap_err();
        assert_eq!(
            err,
            HelperError::CollidingCredentials {
                first: "client-id".into(),
                second: "client.id".into(),
                variable: "DAM_T_CLIENT_ID".into(),
            }
        );
    }

    /// Git's invocation: the remote's name, then the address its url names. A
    /// helper reads its credentials under that name, so the remote is named
    /// apart from its helper here.
    #[test]
    fn the_helper_is_given_its_remote_name_and_address() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[\"%s\",\"%s\",\"%s\",\"%s\"],\"incremental\":false}\\n' \"$#\" \"$1\" \"$2\" \"$DAM_WORK_API_TOKEN\"\n",
        );
        let mut remote = remote();
        remote.name = RemoteName("work".into());
        remote.url = "t::primary,team@x".into();
        let mut helper = launcher
            .launch(&remote, &[("api_token".into(), "tok".into())])
            .unwrap();
        assert_eq!(
            helper.capabilities().unwrap().credentials,
            vec![
                "2".to_string(),
                "work".to_string(),
                "primary,team@x".to_string(),
                "tok".to_string()
            ]
        );
    }

    #[test]
    fn an_empty_address_is_still_passed_so_the_count_never_varies() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\nread -r line\nprintf '{\"protocol\":1,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[\"%s\",\"%s\"],\"incremental\":false}\\n' \"$#\" \"$2\"\n",
        );
        let mut helper = launcher.launch(&remote(), &[]).unwrap();
        assert_eq!(
            helper.capabilities().unwrap().credentials,
            vec!["2".to_string(), String::new()]
        );
    }
}
