use std::cell::RefCell;
use std::rc::Rc;

use dam_application::{ConfigError, TaskTemplate, TaskTemplates, TemplateName, TemplateOwner};

#[derive(Default)]
pub(crate) struct FakeTemplates {
    pub(crate) held: Vec<(String, TemplateOwner, TaskTemplate)>,
    pub(crate) looked_up: Rc<RefCell<Vec<String>>>,
}

impl TaskTemplates for FakeTemplates {
    fn find(&self, name: &TemplateName) -> Result<Option<TaskTemplate>, ConfigError> {
        self.looked_up.borrow_mut().push(name.as_str().into());
        Ok(self
            .held
            .iter()
            .find(|(held, _, _)| held == name.as_str())
            .map(|(_, _, template)| template.clone()))
    }

    fn names(&self) -> Result<Vec<(String, TemplateOwner)>, ConfigError> {
        Ok(self
            .held
            .iter()
            .map(|(name, owner, _)| (name.clone(), *owner))
            .collect())
    }
}
