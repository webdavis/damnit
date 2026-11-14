use std::collections::HashSet;

use serde_json::Value;

use super::Held;

#[derive(Clone, Copy, Debug)]
pub struct Undocumented {
    pub replays_the_temp_id_of_a_skipped_command: bool,
    pub resolves_a_temp_id_from_an_earlier_write: bool,
}

pub const EVERY_WAY_TODOIST_MIGHT_BEHAVE: [Undocumented; 4] = [
    Undocumented::new(true, true),
    Undocumented::new(true, false),
    Undocumented::new(false, true),
    Undocumented::new(false, false),
];

impl Undocumented {
    const fn new(replays: bool, resolves: bool) -> Undocumented {
        Undocumented {
            replays_the_temp_id_of_a_skipped_command: replays,
            resolves_a_temp_id_from_an_earlier_write: resolves,
        }
    }
}

impl Default for Undocumented {
    fn default() -> Undocumented {
        Undocumented::new(true, true)
    }
}

impl Held {
    pub(super) fn resolved(
        &self,
        args: &Value,
        issued_by_this_write: &HashSet<String>,
    ) -> Option<Value> {
        let mut args = args.clone();
        let earlier_writes_count = self.undocumented.resolves_a_temp_id_from_an_earlier_write;
        if let Some(fields) = args.as_object_mut() {
            for value in fields.values_mut() {
                let Some(temp_id) = value.as_str().map(str::to_string) else {
                    continue;
                };
                let Some(id) = self.issued_id_by_temp_id.get(&temp_id) else {
                    continue;
                };
                if !earlier_writes_count && !issued_by_this_write.contains(&temp_id) {
                    return None;
                }
                *value = id.clone().into();
            }
        }
        Some(args)
    }
}
