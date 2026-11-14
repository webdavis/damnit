mod ending;
mod requests;

use std::io::BufReader;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use dam_application::{ConfiguredDuration, HelperError, RemoteConfig};
use dam_protocol::read_line;
use serde_json::Value;

use super::stderr_tail::{StderrTail, keep_the_tail_until_end_of_input};
use crate::Interrupt;

const DEFAULT_ANSWER_DEADLINE: &str = "60s";

fn answer_deadline_when_the_remote_names_none() -> ConfiguredDuration {
    ConfiguredDuration::parse(DEFAULT_ANSWER_DEADLINE).expect("the default deadline parses")
}

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
    deadline: ConfiguredDuration,
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
                .clone()
                .unwrap_or_else(answer_deadline_when_the_remote_names_none),
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
}

#[cfg(test)]
mod tests;
