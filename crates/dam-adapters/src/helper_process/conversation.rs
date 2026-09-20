use std::io::BufReader;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex};
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

/// How much of a helper's standard error is kept for its refusal: the last
/// lines, each cut at a length. A helper writes as much as it likes and dam
/// holds a bounded amount of it.
const STDERR_LINES: usize = 20;
const STDERR_LINE_BYTES: usize = 200;

/// The tail of a helper's standard error, filled by its own reader thread,
/// with the flag that thread sets when it reaches end of input.
#[derive(Clone, Debug, Default)]
struct StderrTail {
    lines: Arc<Mutex<Vec<String>>>,
    at_end: Arc<AtomicBool>,
}

/// Keeps the last `STDERR_LINES` lines the helper wrote, each cut at
/// `STDERR_LINE_BYTES`. Ends at end of input, which is when the child closes
/// its standard error or exits.
fn read_stderr(err: impl std::io::BufRead, tail: &StderrTail) {
    for line in err.lines() {
        let Ok(mut line) = line else { break };
        line.truncate(
            (0..=STDERR_LINE_BYTES.min(line.len()))
                .rev()
                .find(|n| line.is_char_boundary(*n))
                .unwrap_or(0),
        );
        let Ok(mut kept) = tail.lines.lock() else {
            break;
        };
        if kept.len() == STDERR_LINES {
            kept.remove(0);
        }
        kept.push(line);
    }
    tail.at_end.store(true, Ordering::SeqCst);
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
        while !self.stderr_tail.at_end.load(Ordering::SeqCst) && Instant::now() < give_up_at {
            std::thread::sleep(TICK);
        }
        let kept = self.stderr_tail.lines.lock().ok()?;
        (!kept.is_empty()).then(|| kept.join("\n"))
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
mod tests {
    use dam_application::{ConfiguredDuration, HelperError, RemoteHelper};

    use super::super::testing::{install, remote};

    #[test]
    fn a_helper_that_dies_without_answering_quotes_what_it_complained_about() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\necho 'DAM_T_API_TOKEN is not set' >&2\nexit 1\n",
        );
        let mut helper = launcher.spawn(&remote(), &[]).unwrap();
        let err = helper.capabilities().unwrap_err();
        let text = format!("{err:?}");
        assert!(text.contains("DAM_T_API_TOKEN is not set"), "{text}");
    }

    #[test]
    fn a_helper_that_never_answers_carries_its_complaint_into_the_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\necho 'waiting on the upstream api' >&2\nwhile :; do sleep 0.05; done\n",
        );
        let mut remote = remote();
        remote.deadline = Some(ConfiguredDuration {
            value: std::time::Duration::from_millis(100),
            text: "100ms".into(),
        });
        let mut helper = launcher.spawn(&remote, &[]).unwrap();
        match helper.capabilities().unwrap_err() {
            HelperError::Timeout { said, .. } => {
                assert_eq!(said.as_deref(), Some("waiting on the upstream api"))
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn only_the_last_lines_of_a_talkative_helper_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\ni=0\nwhile [ $i -lt 200 ]; do echo \"line $i\" >&2; i=$((i+1)); done\nexit 1\n",
        );
        let mut helper = launcher.spawn(&remote(), &[]).unwrap();
        let text = format!("{:?}", helper.capabilities().unwrap_err());
        assert!(text.contains("line 199"), "{text}");
        assert!(!text.contains("line 0\\n"), "{text}");
        assert!(text.len() < 8000, "the tail is bounded, got {}", text.len());
    }

    #[test]
    fn a_response_of_the_wrong_shape_names_the_shape_and_never_the_task_it_carried() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = install(
            dir.path(),
            "#!/bin/sh\nread -r line; printf '{\"objects\":[{\"oid\":\"01\",\"kind\":\"task\",\"subject\":\"call the clinic\",\"body\":\"ask about the referral\"}],\"removed\":[]}\\n'\n",
        );
        let mut helper = launcher.spawn(&remote(), &[]).unwrap();
        let err = helper.capabilities().unwrap_err();
        let text = format!("{err:?}");
        assert!(text.contains("pull"), "{text}");
        assert!(!text.contains("call the clinic"), "{text}");
        assert!(!text.contains("referral"), "{text}");
    }

    #[test]
    fn a_helper_declaring_a_newer_protocol_is_refused_with_both_versions() {
        let dir = tempfile::tempdir().unwrap();
        let newer = dam_protocol::PROTOCOL_VERSION + 1;
        let launcher = install(
            dir.path(),
            &format!(
                "#!/bin/sh\nread -r line; printf '{{\"protocol\":{newer},\"kinds\":[],\"fields\":[],\"credentials\":[],\"incremental\":false}}\\n'\n"
            ),
        );
        let mut helper = launcher.spawn(&remote(), &[]).unwrap();
        assert_eq!(
            helper.capabilities().unwrap_err(),
            HelperError::UnsupportedProtocol {
                helper: "t".into(),
                found: newer,
                supported: dam_protocol::PROTOCOL_VERSION,
            }
        );
    }

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
                deadline: "100ms".into(),
                said: None
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
                deadline: "2m".into(),
                said: None
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
