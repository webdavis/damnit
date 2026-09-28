use std::io::BufReader;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

use dam_application::{
    HelperError, MutationOutcome, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    RemoteMutation,
};
use dam_protocol::{Request, Response, read_line, write_line};

use super::stderr_tail::{StderrTail, keep_the_tail_until_end_of_input};
use crate::wire::{capabilities_from_wire, mutation_to_wire, outcomes_from_wire, pull_from_wire};

const ANSWER_DEADLINE_WHEN_THE_REMOTE_NAMES_NONE: Duration = Duration::from_secs(60);
const ANSWER_DEADLINE_WHEN_THE_REMOTE_NAMES_NONE_AS_WRITTEN: &str = "60s";

const RECHECK_INTERVAL: Duration = Duration::from_millis(25);

const EXIT_GRACE_BEFORE_THE_KILL: Duration = Duration::from_millis(200);

fn forward_responses_until_one_is_not_a_response(
    mut out: impl std::io::BufRead,
    tx: &Sender<std::io::Result<Option<Response>>>,
) {
    loop {
        let line = read_line::<Response>(&mut out);
        let more = matches!(line, Ok(Some(_)));
        if tx.send(line).is_err() || !more {
            return;
        }
    }
}

#[derive(Debug)]
pub(super) struct ProcessHelper {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: Receiver<std::io::Result<Option<Response>>>,
    helper: String,
    stderr_tail: StderrTail,
    deadline: Duration,
    deadline_as_written: String,
    killed: bool,
}

impl ProcessHelper {
    pub(super) fn new(
        child: Child,
        stdin: ChildStdin,
        stdout: ChildStdout,
        stderr: ChildStderr,
        remote: &RemoteConfig,
    ) -> ProcessHelper {
        let (tx, responses) = channel();
        std::thread::spawn(move || {
            forward_responses_until_one_is_not_a_response(BufReader::new(stdout), &tx)
        });
        let stderr_tail = StderrTail::default();
        let filling = stderr_tail.clone();
        std::thread::spawn(move || {
            keep_the_tail_until_end_of_input(BufReader::new(stderr), &filling)
        });
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
                .unwrap_or(ANSWER_DEADLINE_WHEN_THE_REMOTE_NAMES_NONE),
            deadline_as_written: remote
                .deadline
                .as_ref()
                .map(|d| d.text.clone())
                .unwrap_or_else(|| {
                    ANSWER_DEADLINE_WHEN_THE_REMOTE_NAMES_NONE_AS_WRITTEN.to_string()
                }),
            killed: false,
        }
    }

    fn said(&self) -> Option<String> {
        self.let_an_ending_helpers_last_lines_arrive();
        self.stderr_tail.text()
    }

    fn let_an_ending_helpers_last_lines_arrive(&self) {
        let give_up_at = Instant::now() + EXIT_GRACE_BEFORE_THE_KILL;
        while !self.stderr_tail.reader_reached_end_of_input() && Instant::now() < give_up_at {
            std::thread::sleep(RECHECK_INTERVAL);
        }
    }

    fn io_with_the_helpers_complaint(&self, what: &str) -> HelperError {
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
            return Err(self.io_with_the_helpers_complaint(&e.to_string()));
        }
        self.receive_within_the_deadline()
    }

    fn receive_within_the_deadline(&mut self) -> Result<Response, HelperError> {
        let started = Instant::now();
        loop {
            match self.responses.recv_timeout(RECHECK_INTERVAL) {
                Ok(Ok(Some(Response::Error { error }))) => return Err(HelperError::Remote(error)),
                Ok(Ok(Some(other))) => return Ok(other),
                Ok(Ok(None)) | Err(RecvTimeoutError::Disconnected) => {
                    return Err(self.io_with_the_helpers_complaint("the helper closed its output"));
                }
                Ok(Err(e)) => {
                    return Err(match e.kind() {
                        std::io::ErrorKind::InvalidData => HelperError::Protocol(e.to_string()),
                        _ => self.io_with_the_helpers_complaint(&e.to_string()),
                    });
                }
                Err(RecvTimeoutError::Timeout) => {
                    if crate::cancel::cancellation_requested() {
                        self.kill_and_reap();
                        return Err(HelperError::Cancelled);
                    }
                    if started.elapsed() >= self.deadline {
                        self.kill_and_reap();
                        return Err(HelperError::Timeout {
                            helper: self.helper.clone(),
                            deadline: self.deadline_as_written.clone(),
                            said: self.said(),
                        });
                    }
                }
            }
        }
    }

    fn kill_and_reap(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.killed = true;
    }

    fn close_input_so_a_well_behaved_helper_exits(&mut self) {
        drop(self.stdin.take());
    }

    fn ended_or_unwaitable_within_the_exit_grace(&mut self) -> bool {
        let give_up_at = Instant::now() + EXIT_GRACE_BEFORE_THE_KILL;
        while Instant::now() < give_up_at {
            match self.child.try_wait() {
                Ok(None) => std::thread::sleep(RECHECK_INTERVAL),
                Ok(Some(_)) | Err(_) => return true,
            }
        }
        false
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
        self.close_input_so_a_well_behaved_helper_exits();
        if self.killed || self.ended_or_unwaitable_within_the_exit_grace() {
            return;
        }
        self.kill_and_reap();
    }
}

#[cfg(test)]
mod tests;
