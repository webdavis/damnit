use dam_domain::{Date, FOCUS_PER_DAY, Focus, Object};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn mark_focus(
        &self,
        object: &mut Object,
        day: Option<Date>,
    ) -> Result<(), UseCaseError> {
        let Object::Task(task) = object else {
            return Ok(());
        };
        let Some(day) = day else {
            task.focus = None;
            return Ok(());
        };
        if task.focus_on(day).is_some() {
            return Ok(());
        }
        let mut others: Vec<(Focus, dam_domain::Oid)> = self
            .store
            .all()?
            .into_iter()
            .filter_map(|o| match o {
                Object::Task(t) if t.base.oid != task.base.oid => {
                    t.focus_on(day).cloned().map(|f| (f, t.base.oid))
                }
                _ => None,
            })
            .collect();
        if others.len() >= FOCUS_PER_DAY {
            others.sort_by_key(|(f, _)| f.set_at);
            let focused = others.into_iter().map(|(_, oid)| oid).collect();
            return Err(Refusal::FocusFull { day, focused }.into());
        }
        task.focus = Some(Focus {
            day,
            set_by: self.access.token().unwrap_or("owner").to_string(),
            set_at: self.clock.now(),
        });
        Ok(())
    }
}
