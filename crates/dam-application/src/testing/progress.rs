use std::cell::RefCell;

use crate::ports::{Progress, Step};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Said {
    Started(String, Option<usize>),
    Advanced(usize),
    Finished,
}

#[derive(Default)]
pub struct FakeProgress {
    pub said: RefCell<Vec<Said>>,
}

impl Progress for FakeProgress {
    fn start(&self, step: Step<'_>) {
        let started = Said::Started(format!("{} {}", step.name, step.unit), step.total);
        self.said.borrow_mut().push(started);
    }

    fn advance(&self, done: usize) {
        self.said.borrow_mut().push(Said::Advanced(done));
    }

    fn finish(&self) {
        self.said.borrow_mut().push(Said::Finished);
    }
}
