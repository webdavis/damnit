use std::collections::HashSet;
use std::ffi::c_void;

use mlua::{Table, Value};

const DEEPEST_READ: usize = 4;
const ENTRIES_READ: usize = 10_000;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Walked {
    Clean,
    HoldsAControlCharacter,
    TooLarge,
}

pub(super) fn walked(table: &Table) -> Walked {
    let mut walk = Walk {
        entered: HashSet::new(),
        entries: 0,
    };
    match walk.within(table, 0) {
        Ok(false) => Walked::Clean,
        Ok(true) => Walked::HoldsAControlCharacter,
        Err(TooLarge) => Walked::TooLarge,
    }
}

struct TooLarge;

struct Walk {
    entered: HashSet<*const c_void>,
    entries: usize,
}

impl Walk {
    fn within(&mut self, table: &Table, depth: usize) -> Result<bool, TooLarge> {
        if !self.entered.insert(table.to_pointer()) {
            return Ok(false);
        }
        for pair in table.pairs::<Value, Value>() {
            self.entries += 1;
            if self.entries > ENTRIES_READ {
                return Err(TooLarge);
            }
            let Ok((key, value)) = pair else {
                return Err(TooLarge);
            };
            if self.controlled(&key, depth)? || self.controlled(&value, depth)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn controlled(&mut self, value: &Value, depth: usize) -> Result<bool, TooLarge> {
        match value {
            Value::String(text) => Ok(text.to_string_lossy().chars().any(char::is_control)),
            Value::Table(table) if depth < DEEPEST_READ => self.within(table, depth + 1),
            _ => Ok(false),
        }
    }
}
