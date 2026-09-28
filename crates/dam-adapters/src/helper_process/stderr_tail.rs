use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const LAST_LINES_KEPT: usize = 20;
const BYTES_KEPT_PER_LINE: usize = 200;

#[derive(Clone, Debug, Default)]
pub(super) struct StderrTail {
    lines: Arc<Mutex<Vec<String>>>,
    reader_reached_end_of_input: Arc<AtomicBool>,
}

pub(super) fn keep_the_tail_until_end_of_input(err: impl std::io::BufRead, tail: &StderrTail) {
    for line in err.lines() {
        let Ok(mut line) = line else { break };
        cut_at_a_char_boundary_within(&mut line, BYTES_KEPT_PER_LINE);
        let Ok(mut kept) = tail.lines.lock() else {
            break;
        };
        if kept.len() == LAST_LINES_KEPT {
            kept.remove(0);
        }
        kept.push(line);
    }
    tail.reader_reached_end_of_input
        .store(true, Ordering::SeqCst);
}

fn cut_at_a_char_boundary_within(line: &mut String, bytes: usize) {
    line.truncate(
        (0..=bytes.min(line.len()))
            .rev()
            .find(|n| line.is_char_boundary(*n))
            .unwrap_or(0),
    );
}

impl StderrTail {
    pub(super) fn reader_reached_end_of_input(&self) -> bool {
        self.reader_reached_end_of_input.load(Ordering::SeqCst)
    }

    pub(super) fn text(&self) -> Option<String> {
        let kept = self.lines.lock().ok()?;
        (!kept.is_empty()).then(|| kept.join("\n"))
    }
}
