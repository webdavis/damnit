use std::io::BufReader;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

use dam_application::{
    HelperError, MutationOutcome, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    RemoteMutation,
};
use dam_protocol::{Request, Response, read_line, write_line};

use super::stderr_tail::{StderrTail, read_stderr};
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
    stderr_tail: StderrTail,
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
        stderr: ChildStderr,
        remote: &RemoteConfig,
    ) -> ProcessHelper {
        let (tx, responses) = channel();
        std::thread::spawn(move || read_responses(BufReader::new(stdout), &tx));
        let stderr_tail = StderrTail::default();
        let filling = stderr_tail.clone();
        std::thread::spawn(move || read_stderr(BufReader::new(stderr), &filling));
        ProcessHelper {
            child,
            stdin: Some(stdin),
            responses,
            helper: remote.helper.clone(),
            stderr_tail,
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

    /// What the helper wrote to its standard error, for a refusal that has no
    /// answer of its own to quote.
    ///
    /// A helper that has ended writes its last lines at about the moment its
    /// output closes, so this waits a grace for its reader to reach end of
    /// input. A helper still running keeps its standard error open and there
    /// is nothing to wait for, which is what bounds the wait.
    fn said(&self) -> Option<String> {
        let give_up_at = Instant::now() + EXIT_GRACE;
        while !self.stderr_tail.at_end() && Instant::now() < give_up_at {
            std::thread::sleep(TICK);
        }
        self.stderr_tail.text()
    }

    /// An io failure with the helper's own complaint attached, so the operator
    /// reads why the child gave up rather than reconstructing it.
    fn io(&self, what: &str) -> HelperError {
        match self.said() {
            Some(text) => HelperError::Io(format!("{what}; the helper said: {text}")),
            None => HelperError::Io(what.to_string()),
        }
    }

    fn call(&mut self, request: &Request) -> Result<Response, HelperError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| HelperError::Io("the helper's input is closed".into()))?;
        let written = write_line(stdin, request);
        if let Err(e) = written {
            return Err(self.io(&e.to_string()));
        }
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
                    return Err(self.io("the helper closed its output"));
                }
                Ok(Err(e)) => {
                    return Err(match e.kind() {
                        std::io::ErrorKind::InvalidData => HelperError::Protocol(e.to_string()),
                        _ => self.io(&e.to_string()),
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
                            said: self.said(),
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
            Response::Capabilities(caps) if !caps.supported() => {
                Err(HelperError::UnsupportedProtocol {
                    helper: self.helper.clone(),
                    found: caps.protocol,
                    supported: dam_protocol::PROTOCOL_VERSION,
                })
            }
            Response::Capabilities(caps) => Ok(capabilities_from_wire(&caps)),
            other => Err(HelperError::Protocol(format!(
                "expected a capabilities response, got a {} response",
                other.shape()
            ))),
        }
    }

    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError> {
        match self.call(&Request::Pull {
            since: since.map(str::to_string),
        })? {
            Response::Pull(pull) => Ok(pull_from_wire(pull)),
            other => Err(HelperError::Protocol(format!(
                "expected a pull response, got a {} response",
                other.shape()
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
                "expected a push response, got a {} response",
                other.shape()
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
mod tests;
