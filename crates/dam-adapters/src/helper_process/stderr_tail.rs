//! The tail of what a helper wrote to its standard error, so a refusal with
//! no answer of its own to quote can say what the helper complained about.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// How much of a helper's standard error is kept for its refusal: the last
/// lines, each cut at a length. A helper writes as much as it likes and dam
/// holds a bounded amount of it.
const STDERR_LINES: usize = 20;
const STDERR_LINE_BYTES: usize = 200;

/// The tail of a helper's standard error, filled by its own reader thread,
/// with the flag that thread sets when it reaches end of input.
#[derive(Clone, Debug, Default)]
pub(super) struct StderrTail {
    lines: Arc<Mutex<Vec<String>>>,
    at_end: Arc<AtomicBool>,
}

/// Keeps the last `STDERR_LINES` lines the helper wrote, each cut at
/// `STDERR_LINE_BYTES`. Ends at end of input, which is when the child closes
/// its standard error or exits.
pub(super) fn read_stderr(err: impl std::io::BufRead, tail: &StderrTail) {
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

impl StderrTail {
    /// Whether the reader has reached end of input, which a helper that has
    /// ended reaches at about the moment its output closes.
    pub(super) fn at_end(&self) -> bool {
        self.at_end.load(Ordering::SeqCst)
    }

    /// Every line kept, joined, or `None` when the helper wrote nothing.
    pub(super) fn text(&self) -> Option<String> {
        let kept = self.lines.lock().ok()?;
        (!kept.is_empty()).then(|| kept.join("\n"))
    }
}
