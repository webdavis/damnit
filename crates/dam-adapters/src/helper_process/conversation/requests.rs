use std::sync::mpsc::RecvTimeoutError;
use std::time::Instant;

use dam_application::{
    HelperError, PullOutcome, PushOutcome, RemoteCapabilities, RemoteHelper, RemoteMutation,
};
use dam_protocol::{Capabilities, PullResponse, Reply, Request, write_line};
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{ProcessHelper, RECHECK_INTERVAL};
use crate::wire::{capabilities_from_wire, declared_on_the_wire, mutation_to_wire, pull_from_wire};

impl ProcessHelper {
    fn call<T: DeserializeOwned>(&mut self, request: &Request) -> Result<T, HelperError> {
        self.send(request)?;
        reply_from(self.receive_within_the_deadline()?)
    }

    pub(super) fn send(&mut self, request: &Request) -> Result<(), HelperError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| HelperError::Io("the helper's input is closed".into()))?;
        let written = write_line(stdin, request);
        written.map_err(|e| self.io_with_the_helpers_complaint(&e.to_string()))
    }

    pub(super) fn receive_within_the_deadline(&mut self) -> Result<Value, HelperError> {
        let started = Instant::now();
        loop {
            if self.interrupt.requested() {
                self.kill_and_reap();
                return Err(HelperError::Cancelled);
            }
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
                    if started.elapsed() >= self.deadline.value {
                        self.kill_and_reap();
                        return Err(HelperError::Timeout {
                            helper: self.helper.clone(),
                            deadline: self.deadline.text.clone(),
                            said: self.said(),
                        });
                    }
                }
            }
        }
    }
}

pub(super) fn reply_from<T: DeserializeOwned>(line: Value) -> Result<T, HelperError> {
    match serde_json::from_value(line) {
        Ok(Reply::Answered(answer)) => Ok(answer),
        Ok(Reply::Failed { error }) => Err(HelperError::Remote(error)),
        Err(e) => Err(HelperError::Protocol(e.to_string())),
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

    fn push(&mut self, mutations: Vec<RemoteMutation>) -> Result<PushOutcome, HelperError> {
        self.send(&Request::Push {
            mutations: mutations.into_iter().map(mutation_to_wire).collect(),
            early_answers: true,
        })?;
        Ok(self.read_answers_as_they_come())
    }

    fn declared_on_the_wire(&self, mutation: &RemoteMutation) -> String {
        declared_on_the_wire(mutation)
    }
}
