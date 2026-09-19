use std::ffi::OsString;
use std::io::BufReader;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

use dam_application::{HelperError, HelperLauncher, RemoteConfig, RemoteHelper};
use dam_protocol::{
    Capabilities, Mutation, PullResponse, PushResponse, Request, Response, read_line, write_line,
};

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

/// How long a helper may take to answer one request when the remote names no deadline.
const DEFAULT_DEADLINE: Duration = Duration::from_secs(60);

/// How often the wait loop wakes to re-check the deadline.
const TICK: Duration = Duration::from_millis(25);

impl HelperLauncher for ProcessLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, String)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Ok(Box::new(self.spawn(remote, credentials)?))
    }
}

impl ProcessLauncher {
    fn spawn(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, String)],
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
            .stderr(Stdio::inherit());
        for (name, value) in credentials {
            command.env(credential_variable(&remote.name.0, name), value);
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
        let (tx, responses) = channel();
        std::thread::spawn(move || read_responses(BufReader::new(stdout), &tx));
        Ok(ProcessHelper {
            child,
            stdin: Some(stdin),
            responses,
            helper: remote.helper.clone(),
            deadline: remote.deadline.unwrap_or(DEFAULT_DEADLINE),
            killed: false,
        })
    }
}

/// Reads the helper's output on its own thread so a blocked pipe cannot outlast
/// the deadline. Ends at the first line that is not a response, or when the
/// receiver is gone.
fn read_responses(mut out: impl std::io::BufRead, tx: &Sender<std::io::Result<Option<Response>>>) {
    loop {
        let line = read_line::<Response>(&mut out);
        let more = matches!(line, Ok(Some(_)));
        if tx.send(line).is_err() || !more {
            return;
        }
    }
}

/// A `RemoteHelper` that talks to a spawned child over its stdin and stdout pipes.
#[derive(Debug)]
struct ProcessHelper {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: Receiver<std::io::Result<Option<Response>>>,
    helper: String,
    deadline: Duration,
    killed: bool,
}

impl ProcessHelper {
    fn call(&mut self, request: &Request) -> Result<Response, HelperError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| HelperError::Io("the helper's input is closed".into()))?;
        write_line(stdin, request).map_err(|e| HelperError::Io(e.to_string()))?;
        self.receive()
    }

    /// Waits for one response, giving up at the deadline and killing the child.
    fn receive(&mut self) -> Result<Response, HelperError> {
        let started = Instant::now();
        loop {
            match self.responses.recv_timeout(TICK) {
                Ok(Ok(Some(Response::Error { error }))) => return Err(HelperError::Remote(error)),
                Ok(Ok(Some(other))) => return Ok(other),
                Ok(Ok(None)) | Err(RecvTimeoutError::Disconnected) => {
                    return Err(HelperError::Io("the helper closed its output".into()));
                }
                Ok(Err(e)) => {
                    return Err(match e.kind() {
                        std::io::ErrorKind::InvalidData => HelperError::Protocol(e.to_string()),
                        _ => HelperError::Io(e.to_string()),
                    });
                }
                Err(RecvTimeoutError::Timeout) => {
                    if crate::cancel::cancellation_requested() {
                        self.kill();
                        return Err(HelperError::Cancelled);
                    }
                    if started.elapsed() >= self.deadline {
                        self.kill();
                        return Err(HelperError::Timeout {
                            helper: self.helper.clone(),
                            deadline: render_duration(self.deadline),
                        });
                    }
                }
            }
        }
    }

    /// Ends the child and reaps it, so `Drop` has nothing left to wait for.
    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.killed = true;
    }
}

/// A whole number of seconds reads as seconds, anything finer as milliseconds.
fn render_duration(d: Duration) -> String {
    if d.subsec_millis() == 0 && d.as_secs() > 0 {
        format!("{}s", d.as_secs())
    } else {
        format!("{}ms", d.as_millis())
    }
}

impl RemoteHelper for ProcessHelper {
    fn capabilities(&mut self) -> Result<Capabilities, HelperError> {
        match self.call(&Request::Capabilities)? {
            Response::Capabilities(caps) => Ok(caps),
            other => Err(HelperError::Protocol(format!(
                "expected a capabilities response, got {other:?}"
            ))),
        }
    }

    fn pull(&mut self, since: Option<&str>) -> Result<PullResponse, HelperError> {
        match self.call(&Request::Pull {
            since: since.map(str::to_string),
        })? {
            Response::Pull(pull) => Ok(pull),
            other => Err(HelperError::Protocol(format!(
                "expected a pull response, got {other:?}"
            ))),
        }
    }

    fn push(&mut self, mutations: Vec<Mutation>) -> Result<PushResponse, HelperError> {
        match self.call(&Request::Push { mutations })? {
            Response::Push(push) => Ok(push),
            other => Err(HelperError::Protocol(format!(
                "expected a push response, got {other:?}"
            ))),
        }
    }
}

impl Drop for ProcessHelper {
    fn drop(&mut self) {
        // Closing stdin lets a well-behaved helper see EOF and exit on its own.
        drop(self.stdin.take());
        if !self.killed {
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use dam_application::{HelperError, HelperLauncher, RemoteConfig, RemoteName};
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    /// A shell helper that answers capabilities with its DAM_T_API_TOKEN value in `kinds`,
    /// answers pull with no objects, and echoes push mutations as successes.
    const FAKE: &str = r#"#!/bin/sh
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["task","%s"],"fields":["subject"],"credentials":["api_token"],"incremental":false}\n' "$DAM_T_API_TOKEN" ;;
    *'"pull"'*) printf '{"objects":[],"removed":[],"sync":null}\n' ;;
    *'"push"'*) printf '{"results":[{"oid":"x","ok":true,"remote_id":"r1","why":null}]}\n' ;;
    *) printf '{"error":"unknown"}\n' ;;
  esac
done
"#;

    fn install(dir: &std::path::Path, body: &str) -> ProcessLauncher {
        let file = dir.join("dam-remote-t");
        std::fs::write(&file, body).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        ProcessLauncher::with_search_path(dir.as_os_str())
    }

    fn remote() -> RemoteConfig {
        RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![],
            stale: None,
            deadline: None,
            path: None,
        }
    }

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
        assert_eq!(caps.kinds, vec!["task".to_string(), "tok".to_string()]);
        assert!(helper.pull(None).unwrap().objects.is_empty());
        let pushed = helper.push(vec![]).unwrap();
        assert_eq!(pushed.results[0].remote_id.as_deref(), Some("r1"));
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
    fn a_helper_that_never_answers_times_out_and_its_child_is_killed() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), "#!/bin/sh\nsleep 2\n");
        let mut remote = remote();
        remote.deadline = Some(std::time::Duration::from_millis(100));
        let started = std::time::Instant::now();
        let mut helper = launcher.spawn(&remote, &[]).unwrap();
        let err = helper.capabilities().unwrap_err();
        assert_eq!(
            err,
            HelperError::Timeout {
                helper: "t".into(),
                deadline: "100ms".into()
            }
        );
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert!(helper.child.try_wait().unwrap().is_some());
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
