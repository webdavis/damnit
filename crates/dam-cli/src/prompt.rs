use std::cell::OnceCell;
use std::io::{BufRead, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

use crate::error::CliError;

/// How often a wait on the operator wakes to re-check for an interrupt.
const TICK: Duration = Duration::from_millis(25);

/// Asks the operator a question, for verbs like `done --force --interactive` and remote add.
pub trait Prompt {
    fn choose(&self, question: &str, options: &[&str]) -> Result<usize, CliError>;
    fn text(&self, question: &str) -> Result<String, CliError>;
}

/// Installed instead of `TerminalPrompt` for `--json`/`--toon`, so a verb
/// that needs to ask a question fails instead of blocking on stdin.
pub struct RefusingPrompt;

impl Prompt for RefusingPrompt {
    fn choose(&self, _question: &str, _options: &[&str]) -> Result<usize, CliError> {
        Err(refused())
    }

    fn text(&self, _question: &str) -> Result<String, CliError> {
        Err(refused())
    }
}

fn refused() -> CliError {
    CliError::Usage("a question needs an answer; drop --json/--toon to answer interactively".into())
}

#[derive(Default)]
pub struct TerminalPrompt {
    /// Filled by the first question asked, so a run that asks none leaves
    /// stdin to whatever else inherits the terminal.
    lines: OnceCell<Receiver<std::io::Result<String>>>,
}

impl TerminalPrompt {
    /// One answer from the operator. Interrupting is `Cancelled`, and so is
    /// the end of input.
    fn line(&self) -> Result<String, CliError> {
        let lines = self.lines.get_or_init(read_stdin_on_a_thread);
        loop {
            match lines.recv_timeout(TICK) {
                Ok(Ok(line)) => return Ok(line),
                Ok(Err(e)) => return Err(CliError::Io(e.to_string())),
                Err(RecvTimeoutError::Disconnected) => return Err(CliError::Cancelled),
                Err(RecvTimeoutError::Timeout) => {
                    if dam_adapters::cancellation_requested() {
                        return Err(CliError::Cancelled);
                    }
                }
            }
        }
    }
}

/// Reads stdin on its own thread, because the interrupt handler does not break
/// a blocking read and `read_line` retries an interrupted one itself. The
/// thread ends at the end of input, at the first error, or once nobody is
/// listening.
fn read_stdin_on_a_thread() -> Receiver<std::io::Result<String>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        loop {
            let mut line = String::new();
            match std::io::stdin().lock().read_line(&mut line) {
                Ok(0) => return,
                Ok(_) => {
                    if tx.send(Ok(line)).is_err() {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            }
        }
    });
    rx
}

impl Prompt for TerminalPrompt {
    fn choose(&self, question: &str, options: &[&str]) -> Result<usize, CliError> {
        let mut err = std::io::stderr();
        loop {
            writeln!(err, "{question}")?;
            for (i, o) in options.iter().enumerate() {
                writeln!(err, "  {}) {o}", i + 1)?;
            }
            write!(err, "> ")?;
            err.flush()?;
            if let Some(n) = self
                .line()?
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|n| (1..=options.len()).contains(n))
            {
                return Ok(n - 1);
            }
        }
    }

    fn text(&self, question: &str) -> Result<String, CliError> {
        let mut err = std::io::stderr();
        write!(err, "{question} ")?;
        err.flush()?;
        Ok(self.line()?.trim().to_string())
    }
}
