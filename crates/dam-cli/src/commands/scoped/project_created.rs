use dam_application::Landed;
use dam_domain::Path;
use serde_json::json;

use crate::output::Report;

pub(super) fn told(mut made: Report, created: Option<&Landed>, project: &Path) -> Report {
    let (Some(created), Some(data)) = (created, made.data.as_object_mut()) else {
        return made;
    };
    let oid = created.oid();
    data.insert(
        "project_created".into(),
        json!({ "oid": oid.as_str(), "path": project.as_str() }),
    );
    made.standard_error_lines.push(format!(
        "created task {} to hold the project {project}",
        oid.short()
    ));
    made
}
