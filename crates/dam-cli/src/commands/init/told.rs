use super::Initialized;
use crate::view::{Human, View};

impl View for Initialized {
    fn human(&self) -> Human {
        let text = match (&self.path, &self.task, self.tasks_already_there) {
            (None, ..) => format!(
                "{} already makes this folder a project; nothing changed",
                self.file
            ),
            (Some(path), task, _) if !self.created => {
                let mut lines = vec![format!(
                    "{} already makes this folder a project at {path}; created what was missing",
                    self.file
                )];
                lines.extend(task.iter().map(|task| format!("created task {task}")));
                lines.extend(self.sections_created.iter().map(|section| {
                    format!("created section {} as task {}", section.name, section.task)
                }));
                lines.join("\n")
            }
            (Some(path), Some(task), _) => {
                format!("wrote {} with path {path}\ncreated task {task}", self.file)
            }
            (Some(path), None, Some(there)) => format!(
                "wrote {file} with path {path}\n{path} was already there, with {there} {tasks} under it; this folder now shares that place. To use another place, change path in {file}",
                file = self.file,
                tasks = if there == 1 { "task" } else { "tasks" },
            ),
            (Some(path), None, None) => format!("wrote {} with path {path}", self.file),
        };
        Human::Lines(text.lines().map(String::from).collect())
    }
}
