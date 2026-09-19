use std::io::{BufRead, Write};

use crate::error::CliError;

/// Called by the interactive paths Tasks 29 to 31 add (`done --force --interactive`, remote add).
pub trait Prompt {
    fn choose(&self, question: &str, options: &[&str]) -> Result<usize, CliError>;
    fn text(&self, question: &str) -> Result<String, CliError>;
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
            if std::io::stdin().lock().read_line(&mut line)? == 0 {
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
        if std::io::stdin().lock().read_line(&mut line)? == 0 {
            return Err(CliError::Cancelled);
        }
        Ok(line.trim().to_string())
    }
}
