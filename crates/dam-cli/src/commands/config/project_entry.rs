use std::collections::BTreeMap;

use dam_application::Project;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ProjectEntry {
    file: String,
    path: String,
    minimum_dam: Option<String>,
    new_task: NewTaskEntry,
    required: Vec<&'static str>,
    ls: Option<String>,
    filter: BTreeMap<String, FilterEntry>,
}

#[derive(Serialize)]
struct NewTaskEntry {
    labels: Option<Vec<String>>,
    priority: Option<u8>,
    due: Option<String>,
    deadline: Option<String>,
}

#[derive(Serialize)]
struct FilterEntry {
    query: String,
}

impl ProjectEntry {
    pub(super) fn of(project: &Project) -> ProjectEntry {
        let settings = &project.settings;
        let template = &settings.new_task;
        ProjectEntry {
            file: project.file().display().to_string(),
            path: settings.path.clone(),
            minimum_dam: settings.minimum_dam.clone(),
            new_task: NewTaskEntry {
                labels: template.labels.clone(),
                priority: template.priority.map(|priority| priority.get()),
                due: template.due.clone(),
                deadline: template.deadline.clone(),
            },
            required: settings.required.iter().map(|field| field.name()).collect(),
            ls: settings.ls.clone(),
            filter: settings
                .filters
                .iter()
                .filter_map(|filter| {
                    let query = filter.rule.query()?.to_string();
                    Some((filter.name.clone(), FilterEntry { query }))
                })
                .collect(),
        }
    }

    pub(super) fn file(&self) -> &str {
        &self.file
    }

    pub(super) fn rows(&self) -> Vec<(String, String)> {
        let task = &self.new_task;
        let mut rows = vec![("path".to_string(), Some(self.path.clone()))];
        rows.push(("minimum_dam".into(), self.minimum_dam.clone()));
        rows.push((
            "new_task.labels".into(),
            task.labels.as_ref().map(|l| l.join(", ")),
        ));
        rows.push((
            "new_task.priority".into(),
            task.priority.map(|p| p.to_string()),
        ));
        rows.push(("new_task.due".into(), task.due.clone()));
        rows.push(("new_task.deadline".into(), task.deadline.clone()));
        let required = (!self.required.is_empty()).then(|| self.required.join(", "));
        rows.push(("required".into(), required));
        rows.push(("ls".into(), self.ls.clone()));
        for (name, filter) in &self.filter {
            rows.push((format!("filter.{name}.query"), Some(filter.query.clone())));
        }
        rows.into_iter()
            .filter_map(|(key, value)| Some((key, format!("{} (from the project)", value?))))
            .collect()
    }
}
