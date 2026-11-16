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
    category: BTreeMap<String, CategoryEntry>,
    sections: Vec<String>,
    branch_label: bool,
    hooks: bool,
    hooks_from: &'static str,
    done_children: Option<String>,
}

#[derive(Serialize)]
struct CategoryEntry {
    values: Vec<String>,
    exclusive: bool,
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
            category: settings
                .categories
                .all()
                .iter()
                .map(|category| {
                    let entry = CategoryEntry {
                        values: category.values.clone(),
                        exclusive: category.exclusive,
                    };
                    (category.name.clone(), entry)
                })
                .collect(),
            sections: settings.sections.clone(),
            branch_label: settings.branch_label,
            hooks: false,
            hooks_from: "default",
            done_children: settings.done_children.as_ref().map(|d| d.words()),
        }
    }

    pub(super) fn with_hooks(self, hooks: bool, hooks_from: &'static str) -> ProjectEntry {
        ProjectEntry {
            hooks,
            hooks_from,
            ..self
        }
    }

    pub(super) fn hooks_line(&self) -> String {
        let on = if self.hooks { "on" } else { "off" };
        format!("{on} (from {})", self.hooks_from)
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
        for (name, category) in &self.category {
            rows.push((
                format!("category.{name}.values"),
                Some(category.values.join(", ")),
            ));
            rows.push((
                format!("category.{name}.exclusive"),
                Some(category.exclusive.to_string()),
            ));
        }
        let sections = (!self.sections.is_empty()).then(|| self.sections.join(", "));
        rows.push(("sections".into(), sections));
        let branch_label = self.branch_label.then(|| "true".to_string());
        rows.push(("branch_label".into(), branch_label));
        rows.push(("done_children".into(), self.done_children.clone()));
        rows.into_iter()
            .filter_map(|(key, value)| Some((key, format!("{} (from the project)", value?))))
            .collect()
    }
}
