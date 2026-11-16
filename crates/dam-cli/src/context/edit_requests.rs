use dam_application::{EditFields, EditRequest, EditRequests, Refusal};
use dam_domain::Oid;

use super::Context;
use crate::commands::each::one_object;
use crate::commands::edit::from_flags;
use crate::error::{RefusalText, UseCaseText};
use crate::request::hook_edit_args;

impl EditRequests for Context {
    fn fields(&self, task: &Oid, request: &EditRequest) -> Result<EditFields, String> {
        let args = hook_edit_args(task, &request.json).map_err(|e| e.to_string())?;
        let Some(oid) = one_object(&args.oid, &args.set).filter(|_| args.set.is_plain()) else {
            return Err(
                "--request: a hook's request changes only the task its hook ran for".into(),
            );
        };
        let named = self
            .dam()
            .resolve_object(oid)
            .map_err(|e| format!("--request: oid: {}", UseCaseText(&e)))?;
        if &named != task {
            return Err(format!(
                "--request: oid names {}, not the task its hook ran for",
                named.short()
            ));
        }
        from_flags(self, args).map_err(|e| e.to_string())
    }

    fn refused(&self, refusal: &Refusal) -> String {
        RefusalText(refusal).to_string()
    }
}
