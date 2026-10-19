mod ending;

use std::io::BufReader;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

use dam_application::{
    HelperError, MutationOutcome, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    RemoteMutation,
};
use dam_protocol::{
    Capabilities, PullResponse, PushResponse, Reply, Request, read_line, write_line,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::stderr_tail::{StderrTail, keep_the_tail_until_end_of_input};
use crate::Interrupt;
use crate::wire::{capabilities_from_wire, mutation_to_wire, outcomes_from_wire, pull_from_wire};

const ANSWER_DEADLINE_WHEN_THE_REMOTE_NAMES_NONE: Duration = Duration::from_secs(60);
const ANSWER_DEADLINE_WHEN_THE_REMOTE_NAMES_NONE_AS_WRITTEN: &str = "60s";

const RECHECK_INTERVAL: Duration = Duration::from_millis(25);

const EXIT_GRACE_BEFORE_THE_KILL: Duration = Duration::from_millis(200);

fn forward_answers_until_one_is_not_json(
    mut out: impl std::io::BufRead,
    tx: &Sender<std::io::Result<Option<Value>>>,
) {
    loop {
        let line = read_line::<Value>(&mut out);
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
    answers: Receiver<std::io::Result<Option<Value>>>,
    helper: String,
    stderr_tail: StderrTail,
    deadline: Duration,
    deadline_as_written: String,
    killed: bool,
    interrupt: Interrupt,
}

impl ProcessHelper {
    pub(super) fn new(
        child: Child,
        stdin: ChildStdin,
        stdout: ChildStdout,
        stderr: ChildStderr,
        remote: &RemoteConfig,
        interrupt: Interrupt,
    ) -> ProcessHelper {
        let (tx, answers) = channel();
        std::thread::spawn(move || {
            forward_answers_until_one_is_not_json(BufReader::new(stdout), &tx)
        });
        let stderr_tail = StderrTail::default();
        let filling = stderr_tail.clone();
        std::thread::spawn(move || {
            keep_the_tail_until_end_of_input(BufReader::new(stderr), &filling)
        });
        ProcessHelper {
            child,
            stdin: Some(stdin),
            answers,
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
            interrupt,
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

    fn call<T: DeserializeOwned>(&mut self, request: &Request) -> Result<T, HelperError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| HelperError::Io("the helper's input is closed".into()))?;
        let written = write_line(stdin, request);
        if let Err(e) = written {
            return Err(self.io_with_the_helpers_complaint(&e.to_string()));
        }
        match serde_json::from_value(self.receive_within_the_deadline()?) {
            Ok(Reply::Answered(answer)) => Ok(answer),
            Ok(Reply::Failed { error }) => Err(HelperError::Remote(error)),
            Err(e) => Err(HelperError::Protocol(e.to_string())),
        }
    }

    fn receive_within_the_deadline(&mut self) -> Result<Value, HelperError> {
        let started = Instant::now();
        loop {
            match self.answers.recv_timeout(RECHECK_INTERVAL) {
                Ok(Ok(Some(answer))) => return Ok(answer),
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
                    if self.interrupt.requested() {
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
}

impl RemoteHelper for ProcessHelper {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError> {
        let caps: Capabilities = self.call(&Request::Capabilities)?;
        if !caps.supported() {
            return Err(HelperError::UnsupportedProtocol {
                helper: self.helper.clone(),
                found: caps.protocol,
                supported: dam_protocol::PROTOCOL_VERSION,
            });
        }
        Ok(capabilities_from_wire(&caps))
    }

    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError> {
        let pulled: PullResponse = self.call(&Request::Pull {
            since: since.map(str::to_string),
        })?;
        Ok(pull_from_wire(pulled))
    }

    fn push(
        &mut self,
        mutations: Vec<RemoteMutation>,
    ) -> Result<Vec<MutationOutcome>, HelperError> {
        let mutations = mutations.into_iter().map(mutation_to_wire).collect();
        let pushed: PushResponse = self.call(&Request::Push { mutations })?;
        Ok(outcomes_from_wire(pushed.results))
    }
}

#[cfg(test)]
mod tests;
