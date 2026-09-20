use std::io::BufReader;
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

use dam_application::{
    HelperError, MutationOutcome, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    RemoteMutation,
};
use dam_protocol::{Request, Response, read_line, write_line};

use crate::wire::{capabilities_from_wire, mutation_to_wire, outcomes_from_wire, pull_from_wire};

/// How long a helper may take to answer one request when the remote names no
/// deadline, and the text the message uses for it.
const DEFAULT_DEADLINE: Duration = Duration::from_secs(60);
const DEFAULT_DEADLINE_TEXT: &str = "60s";

/// How often the wait loop wakes to re-check the deadline.
const TICK: Duration = Duration::from_millis(25);

/// How long a helper gets to exit on end of input before it is killed.
const EXIT_GRACE: Duration = Duration::from_millis(200);

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
pub(super) struct ProcessHelper {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: Receiver<std::io::Result<Option<Response>>>,
    helper: String,
    deadline: Duration,
    /// The deadline as the operator wrote it, for the message.
    deadline_text: String,
    killed: bool,
}

impl ProcessHelper {
    /// Takes the spawned child's pipes and starts the thread that reads its output.
    pub(super) fn new(
        child: Child,
        stdin: ChildStdin,
        stdout: ChildStdout,
        remote: &RemoteConfig,
    ) -> ProcessHelper {
        let (tx, responses) = channel();
        std::thread::spawn(move || read_responses(BufReader::new(stdout), &tx));
        ProcessHelper {
            child,
            stdin: Some(stdin),
            responses,
            helper: remote.helper.clone(),
            deadline: remote
                .deadline
                .as_ref()
                .map(|d| d.value)
                .unwrap_or(DEFAULT_DEADLINE),
            deadline_text: remote
                .deadline
                .as_ref()
                .map(|d| d.text.clone())
                .unwrap_or_else(|| DEFAULT_DEADLINE_TEXT.to_string()),
            killed: false,
        }
    }

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
                            deadline: self.deadline_text.clone(),
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

impl RemoteHelper for ProcessHelper {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError> {
        match self.call(&Request::Capabilities)? {
            Response::Capabilities(caps) => Ok(capabilities_from_wire(&caps)),
            other => Err(HelperError::Protocol(format!(
                "expected a capabilities response, got {other:?}"
            ))),
        }
    }

    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError> {
        match self.call(&Request::Pull {
            since: since.map(str::to_string),
        })? {
            Response::Pull(pull) => Ok(pull_from_wire(pull)),
            other => Err(HelperError::Protocol(format!(
                "expected a pull response, got {other:?}"
            ))),
        }
    }

    fn push(
        &mut self,
        mutations: Vec<RemoteMutation>,
    ) -> Result<Vec<MutationOutcome>, HelperError> {
        let mutations = mutations.into_iter().map(mutation_to_wire).collect();
        match self.call(&Request::Push { mutations })? {
            Response::Push(push) => Ok(outcomes_from_wire(push.results)),
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
        if self.killed {
            return;
        }
        let give_up_at = Instant::now() + EXIT_GRACE;
        while Instant::now() < give_up_at {
            match self.child.try_wait() {
                Ok(None) => std::thread::sleep(TICK),
                // Gone, or unwaitable; either way there is nothing left to do.
                Ok(Some(_)) | Err(_) => return,
            }
        }
        self.kill();
    }
}

#[cfg(test)]
mod tests {
    use dam_application::{ConfiguredDuration, HelperError, RemoteHelper};

    use super::super::testing::{install, remote};

    #[test]
    fn a_helper_that_never_answers_times_out_and_its_child_is_killed() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), "#!/bin/sh\nsleep 2\n");
        let mut remote = remote();
        remote.deadline = Some(ConfiguredDuration {
            value: std::time::Duration::from_millis(100),
            text: "100ms".into(),
        });
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
    fn the_timeout_echoes_the_configured_text_rather_than_seconds() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), "#!/bin/sh\nwhile :; do sleep 0.05; done\n");
        let mut remote = remote();
        remote.deadline = Some(ConfiguredDuration {
            value: std::time::Duration::from_millis(50),
            text: "2m".into(),
        });
        let mut helper = launcher.spawn(&remote, &[]).unwrap();
        assert_eq!(
            helper.capabilities().unwrap_err(),
            HelperError::Timeout {
                helper: "t".into(),
                deadline: "2m".into()
            }
        );
    }

    #[test]
    fn a_helper_that_ignores_end_of_input_is_killed_at_drop() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(dir.path(), "#!/bin/sh\nwhile :; do sleep 0.05; done\n");
        let helper = launcher.spawn(&remote(), &[]).unwrap();
        let pid = helper.child.id().to_string();
        // Dropped on a worker so a regression to an unbounded wait reddens the
        // suite within the second instead of hanging it.
        let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = std::sync::Arc::clone(&dropped);
        std::thread::spawn(move || {
            drop(helper);
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        let give_up_at = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while !dropped.load(std::sync::atomic::Ordering::SeqCst) {
            if std::time::Instant::now() >= give_up_at {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid])
                    .status();
                panic!("the launcher was still dropping a second later");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            !std::process::Command::new("kill")
                .args(["-0", &pid])
                .output()
                .unwrap()
                .status
                .success(),
            "the helper outlived its launcher"
        );
    }
}
