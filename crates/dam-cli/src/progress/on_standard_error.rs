use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use dam_adapters::SystemClock;
use dam_application::{Clock, Progress, Step};
use dam_domain::Timestamp;

use super::line::{Moment, Stage, line};
use super::shown::Shown;
use crate::output::Format;

const REDRAW_EVERY: Duration = Duration::from_millis(100);
const CLEAR_THE_LINE: &str = "\r\x1b[2K";

struct Running {
    name: String,
    unit: String,
    total: Option<usize>,
    done: usize,
    started: Timestamp,
}

impl Running {
    fn said(&self, stage: Stage, format: Format) -> String {
        let seconds = SystemClock.now().duration_since(self.started).as_secs();
        let moment = Moment {
            name: &self.name,
            unit: &self.unit,
            done: self.done,
            total: self.total,
            seconds: seconds.max(0).unsigned_abs(),
        };
        line(&moment, stage, format)
    }
}

pub(crate) struct ProgressOnStandardError {
    shown: Shown,
    format: Format,
    running: Arc<Mutex<Option<Running>>>,
    drawing: Arc<AtomicBool>,
    drawer: Mutex<Option<JoinHandle<()>>>,
}

impl ProgressOnStandardError {
    pub(crate) fn new(shown: Shown, format: Format) -> ProgressOnStandardError {
        ProgressOnStandardError {
            shown,
            format,
            running: Arc::default(),
            drawing: Arc::default(),
            drawer: Mutex::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shown(&self) -> Shown {
        self.shown
    }

    fn redraw_until_finished(&self) -> JoinHandle<()> {
        let running = Arc::clone(&self.running);
        let drawing = Arc::clone(&self.drawing);
        let format = self.format;
        std::thread::spawn(move || {
            let mut frame = 0;
            while drawing.load(Ordering::SeqCst) {
                if let Some(step) = running.lock().unwrap().as_ref() {
                    let drawn = step.said(Stage::Running(frame), format);
                    let _ = write!(std::io::stderr(), "{CLEAR_THE_LINE}{drawn}");
                }
                frame += 1;
                std::thread::sleep(REDRAW_EVERY);
            }
        })
    }
}

impl Progress for ProgressOnStandardError {
    fn start(&self, step: Step<'_>) {
        if self.shown == Shown::Hidden {
            return;
        }
        self.finish();
        let running = Running {
            name: step.name.to_string(),
            unit: step.unit.to_string(),
            total: step.total,
            done: 0,
            started: SystemClock.now(),
        };
        if self.shown == Shown::Records {
            eprintln!("{}", running.said(Stage::Started, self.format));
        }
        *self.running.lock().unwrap() = Some(running);
        if self.shown == Shown::Animated {
            self.drawing.store(true, Ordering::SeqCst);
            *self.drawer.lock().unwrap() = Some(self.redraw_until_finished());
        }
    }

    fn advance(&self, done: usize) {
        if let Some(step) = self.running.lock().unwrap().as_mut() {
            step.done = done;
        }
    }

    fn finish(&self) {
        self.drawing.store(false, Ordering::SeqCst);
        if let Some(drawer) = self.drawer.lock().unwrap().take() {
            let _ = drawer.join();
        }
        let Some(step) = self.running.lock().unwrap().take() else {
            return;
        };
        let finished = step.said(Stage::Finished, self.format);
        match self.shown {
            Shown::Animated => eprintln!("{CLEAR_THE_LINE}{finished}"),
            _ => eprintln!("{finished}"),
        }
    }
}

impl Drop for ProgressOnStandardError {
    fn drop(&mut self) {
        self.finish();
    }
}
