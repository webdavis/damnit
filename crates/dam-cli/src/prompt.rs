use std::cell::OnceCell;
use std::io::{BufRead, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

use dam_application::Refusal;

use crate::error::CliError;

const INTERRUPT_RECHECK_INTERVAL: Duration = Duration::from_millis(25);

pub(crate) trait Prompt {
    fn choose(&self, question: &str, options: &[&str]) -> Result<usize, CliError>;
    fn text(&self, question: &str) -> Result<String, CliError>;
}

pub(crate) struct RefusingPrompt;

impl Prompt for RefusingPrompt {
    fn choose(&self, _question: &str, _options: &[&str]) -> Result<usize, CliError> {
        Err(refused())
    }

    fn text(&self, _question: &str) -> Result<String, CliError> {
        Err(refused())
    }
}

fn refused() -> CliError {
    Refusal::NeedsAnAnswer.into()
}

#[derive(Default)]
pub(crate) struct TerminalPrompt {
    stdin_lines_once_the_first_question_is_asked: OnceCell<Receiver<std::io::Result<String>>>,
}

impl TerminalPrompt {
    fn answer_unless_interrupted_or_at_end_of_input(&self) -> Result<String, CliError> {
        let lines = self
            .stdin_lines_once_the_first_question_is_asked
            .get_or_init(read_stdin_on_a_thread_since_an_interrupt_cannot_break_a_blocking_read);
        loop {
            match lines.recv_timeout(INTERRUPT_RECHECK_INTERVAL) {
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

fn read_stdin_on_a_thread_since_an_interrupt_cannot_break_a_blocking_read()
-> Receiver<std::io::Result<String>> {
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
        choose_asking_once(
            &mut std::io::stderr(),
            &mut || self.answer_unless_interrupted_or_at_end_of_input(),
            question,
            options,
        )
    }

    fn text(&self, question: &str) -> Result<String, CliError> {
        let mut err = std::io::stderr();
        write!(err, "{question} ")?;
        err.flush()?;
        Ok(self
            .answer_unless_interrupted_or_at_end_of_input()?
            .trim()
            .to_string())
    }
}

fn choose_asking_once(
    err: &mut dyn Write,
    answer: &mut dyn FnMut() -> Result<String, CliError>,
    question: &str,
    options: &[&str],
) -> Result<usize, CliError> {
    writeln!(err, "{question}")?;
    for (i, o) in options.iter().enumerate() {
        writeln!(err, "  {}) {o}", i + 1)?;
    }
    loop {
        write!(err, "> ")?;
        err.flush()?;
        match answer()?
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|n| (1..=options.len()).contains(n))
        {
            Some(n) => return Ok(n - 1),
            None => writeln!(err, "answer {}", one_of_the_numbers(options.len()))?,
        }
    }
}

fn one_of_the_numbers(count: usize) -> String {
    let numbers: Vec<String> = (1..=count).map(|n| n.to_string()).collect();
    match numbers.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        _ => numbers.concat(),
    }
}

#[cfg(test)]
mod tests;
