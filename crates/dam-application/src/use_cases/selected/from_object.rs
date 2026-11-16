use dam_domain::{Object, WorkedOut, worked_out};

use super::facts::Facts;
use crate::service::Dam;

impl Dam<'_> {
    pub fn worked_out_from_object(&self, object: &Object) -> WorkedOut {
        let categories = self.config.categories_at(&object.base().path);
        let facts = Facts::unread(self, categories);
        WorkedOut {
            blocked: None,
            ..worked_out(object, &facts)
        }
    }
}
