use std::io::{BufRead, Write};

use crate::error::CliError;

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

pub struct TerminalPrompt;

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
            let mut line = String::new();
            let read = std::io::stdin().lock().read_line(&mut line);
            if dam_adapters::cancellation_requested() || read? == 0 {
                return Err(CliError::Cancelled);
            }
            if let Some(n) = line
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
        let mut line = String::new();
        let read = std::io::stdin().lock().read_line(&mut line);
        if dam_adapters::cancellation_requested() || read? == 0 {
            return Err(CliError::Cancelled);
        }
        Ok(line.trim().to_string())
    }
}
