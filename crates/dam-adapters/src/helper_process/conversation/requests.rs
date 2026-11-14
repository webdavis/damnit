use std::sync::mpsc::RecvTimeoutError;
use std::time::Instant;

use dam_application::{
    HelperError, PullOutcome, PushOutcome, RemoteCapabilities, RemoteHelper, RemoteMutation,
};
use dam_protocol::{Capabilities, PullResponse, PushResponse, Reply, Request, write_line};
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{ProcessHelper, RECHECK_INTERVAL};
use crate::wire::{capabilities_from_wire, mutation_to_wire, pull_from_wire, push_from_wire};

impl ProcessHelper {
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
        let mutations = mutations.into_iter().map(mutation_to_wire).collect();
        let pushed: PushResponse = self.call(&Request::Push { mutations })?;
        Ok(push_from_wire(pushed))
    }
}
