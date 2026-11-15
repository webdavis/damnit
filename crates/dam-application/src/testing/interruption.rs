use std::cell::Cell;

use crate::ports::Interruption;

#[derive(Default)]
pub struct FakeInterruption {
    pub pressed: Cell<bool>,
}

impl Interruption for FakeInterruption {
    fn requested(&self) -> bool {
        self.pressed.get()
    }
}
