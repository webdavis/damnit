use crate::api::{Item, Project, Section, SyncResponse};

pub(super) fn project(id: &str, name: &str) -> Project {
    Project {
        id: id.into(),
        name: name.into(),
        parent_id: None,
        is_deleted: false,
        is_archived: false,
        inbox_project: false,
    }
}

pub(super) fn section(id: &str, name: &str) -> Section {
    Section {
        id: id.into(),
        name: name.into(),
        project_id: String::new(),
        is_deleted: false,
    }
}

pub(super) fn item(id: &str, content: &str) -> Item {
    Item {
        id: id.into(),
        content: content.into(),
        description: String::new(),
        project_id: String::new(),
        section_id: None,
        parent_id: None,
        priority: 1,
        due: None,
        deadline: None,
        labels: vec![],
        checked: false,
        is_deleted: false,
    }
}

pub(super) fn sync_of(projects: Vec<Project>, items: Vec<Item>) -> SyncResponse {
    SyncResponse {
        projects,
        sections: vec![],
        items,
    }
}

impl Project {
    pub(super) fn under(mut self, parent: &str) -> Self {
        self.parent_id = Some(parent.into());
        self
    }

    pub(super) fn archived(mut self) -> Self {
        self.is_archived = true;
        self
    }
}

impl Section {
    pub(super) fn under(mut self, project: &str) -> Self {
        self.project_id = project.into();
        self
    }
}

impl Item {
    pub(super) fn under(mut self, project: &str) -> Self {
        self.project_id = project.into();
        self
    }

    pub(super) fn in_section(mut self, section: &str) -> Self {
        self.section_id = Some(section.into());
        self
    }

    pub(super) fn child_of(mut self, parent: &str) -> Self {
        self.parent_id = Some(parent.into());
        self
    }
}
