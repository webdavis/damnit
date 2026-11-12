use dam_domain::CallerId;

use crate::output::Report;

pub(crate) fn with_caller_id(mut report: Report, id: Option<&CallerId>) -> Report {
    if let (Some(id), Some(data)) = (id, report.data.as_object_mut()) {
        data.insert("id".into(), id.as_str().into());
    }
    report
}
