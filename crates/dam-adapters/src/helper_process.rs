mod conversation;
mod stderr_tail;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use dam_application::{HelperError, HelperLauncher, RemoteConfig, RemoteHelper, Secret};
use dam_protocol::credential_variable;

use crate::Interrupt;
use conversation::ProcessHelper;

pub struct ProcessLauncher {
    search_path: OsString,
    interrupt: Interrupt,
}

impl ProcessLauncher {
    pub fn from_env(interrupt: Interrupt) -> ProcessLauncher {
        ProcessLauncher::with_search_path(std::env::var_os("PATH").unwrap_or_default(), interrupt)
    }

    pub fn with_search_path(path: impl Into<OsString>, interrupt: Interrupt) -> ProcessLauncher {
        ProcessLauncher {
            search_path: path.into(),
            interrupt,
        }
    }

    fn find(&self, helper: &str) -> Option<PathBuf> {
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
        let mut child = Command::new(program)
            .arg(&remote.name.0)
            .arg(remote.address())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(credential_environment_refusing_a_shared_variable(
                remote,
                credentials,
            )?)
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
        Ok(ProcessHelper::new(
            child,
            stdin,
            stdout,
            stderr,
            remote,
            self.interrupt.clone(),
        ))
    }
}

fn credential_environment_refusing_a_shared_variable<'a>(
    remote: &RemoteConfig,
    credentials: &'a [(String, Secret)],
) -> Result<Vec<(String, &'a str)>, HelperError> {
    let mut first_name_of: BTreeMap<String, &str> = BTreeMap::new();
    let mut environment = Vec::with_capacity(credentials.len());
    for (name, value) in credentials {
        let variable = credential_variable(&remote.name.0, name);
        if let Some(first) = first_name_of.insert(variable.clone(), name) {
            return Err(HelperError::CollidingCredentials {
                first: first.to_string(),
                second: name.clone(),
                variable,
            });
        }
        environment.push((variable, value.expose()));
    }
    Ok(environment)
}

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;
